//! Worker-owned native counter state. No thread, network traffic, UI or process creation.
use super::NetworkSample;
use std::{
    io,
    time::{Duration, Instant},
};
use windows_sys::Win32::NetworkManagement::{
    IpHelper::{FreeMibTable, GetIfEntry2, GetIfTable2, MIB_IF_ROW2, MIB_IF_TABLE2},
    Ndis::NET_LUID_LH,
};
const MAX_ROWS: usize = 256;
const MAX_GAP: Duration = Duration::from_secs(3);
#[derive(Clone, Copy, PartialEq, Eq)]
struct Identity {
    luid: u64,
    guid: [u8; 16],
}
struct Table(*mut MIB_IF_TABLE2);
impl Drop for Table {
    fn drop(&mut self) {
        // SAFETY: successful API allocation, exactly one owner.
        unsafe { FreeMibTable(self.0.cast()) };
    }
}
#[derive(Clone)]
struct Observed {
    alias: String,
    identity: Identity,
    up: bool,
    received: u64,
    sent: u64,
    at: Instant,
}
pub(super) fn valid_alias(value: &str) -> bool {
    !value.is_empty() && value.encode_utf16().count() <= 256 && !value.chars().any(char::is_control)
}
fn alias(row: &MIB_IF_ROW2) -> io::Result<String> {
    let end = row
        .Alias
        .iter()
        .position(|v| *v == 0)
        .ok_or(io::ErrorKind::InvalidData)?;
    let value = String::from_utf16(&row.Alias[..end]).map_err(|_| io::ErrorKind::InvalidData)?;
    if !valid_alias(&value) {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(value)
}
fn observed(row: &MIB_IF_ROW2) -> io::Result<Observed> {
    let g = row.InterfaceGuid;
    let mut guid = [0; 16];
    guid[..4].copy_from_slice(&g.data1.to_le_bytes());
    guid[4..6].copy_from_slice(&g.data2.to_le_bytes());
    guid[6..8].copy_from_slice(&g.data3.to_le_bytes());
    guid[8..].copy_from_slice(&g.data4);
    // SAFETY: scalar Value representation of the initialized native union.
    let luid = unsafe { row.InterfaceLuid.Value };
    if luid == 0 || guid == [0; 16] {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(Observed {
        alias: alias(row)?,
        identity: Identity { luid, guid },
        up: row.OperStatus == 1,
        received: row.InOctets,
        sent: row.OutOctets,
        at: Instant::now(),
    })
}
fn lookup(selected: &str) -> io::Result<Observed> {
    let mut raw = std::ptr::null_mut(); // SAFETY: valid output pointer; ownership only on API success.
    let code = unsafe { GetIfTable2(&mut raw) };
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    if raw.is_null() {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let table = Table(raw); // SAFETY: API supplied a valid header.
    let count = unsafe { (*table.0).NumEntries } as usize;
    if count > MAX_ROWS {
        return Err(io::ErrorKind::InvalidData.into());
    }
    // SAFETY: generated native offset/stride includes documented table padding.
    let first = unsafe { std::ptr::addr_of!((*table.0).Table).cast::<MIB_IF_ROW2>() };
    let mut chosen = None;
    for index in 0..count {
        // SAFETY: validated NumEntries bounds native array traversal.
        let row = unsafe { &*first.add(index) };
        let name = alias(row)?;
        if name == selected {
            if chosen.is_some() {
                return Err(io::ErrorKind::InvalidData.into());
            }
            chosen = Some(observed(row)?);
        }
    }
    chosen.ok_or_else(|| io::ErrorKind::NotFound.into())
}
fn refresh(binding: &Observed) -> io::Result<Observed> {
    let mut row = MIB_IF_ROW2 {
        InterfaceLuid: NET_LUID_LH {
            Value: binding.identity.luid,
        },
        ..Default::default()
    };
    // SAFETY: zeroed row plus original LUID selector, valid output buffer.
    let code = unsafe { GetIfEntry2(&mut row) };
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    let current = observed(&row)?;
    if current.alias != binding.alias {
        return Err(io::ErrorKind::NotFound.into());
    }
    Ok(current)
}
#[derive(Default)]
pub(super) struct Counter {
    binding: Option<Observed>,
    previous: Option<Observed>,
}
impl Counter {
    pub(super) fn reset(&mut self) {
        self.binding = None;
        self.previous = None;
    }
    fn accept(&mut self, current: Observed) -> Option<NetworkSample> {
        if !current.up {
            self.previous = None;
            return None;
        }
        let discontinuity = self.previous.as_ref().is_some_and(|old| {
            old.identity != current.identity
                || old.alias != current.alias
                || current
                    .at
                    .checked_duration_since(old.at)
                    .is_none_or(|gap| gap.is_zero() || gap > MAX_GAP)
                || current.received < old.received
                || current.sent < old.sent
        });
        let sample = NetworkSample {
            interface: current.alias.clone(),
            received_bytes: current.received,
            sent_bytes: current.sent,
            observed_at: current.at,
        };
        self.previous = Some(current);
        if discontinuity { None } else { Some(sample) }
    }
    pub(super) fn sample(&mut self, selected: &str) -> io::Result<Option<NetworkSample>> {
        if !valid_alias(selected) {
            self.reset();
            return Err(io::ErrorKind::InvalidInput.into());
        }
        if self.binding.as_ref().is_some_and(|b| b.alias != selected) {
            self.reset();
        }
        let result = match &self.binding {
            Some(binding) => refresh(binding),
            None => lookup(selected),
        };
        match result {
            Ok(current) => {
                self.binding = Some(current.clone());
                Ok(self.accept(current))
            }
            Err(error) => {
                self.reset();
                Err(error)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn row(at: Instant, luid: u64, guid: u8, up: bool, received: u64) -> Observed {
        Observed {
            alias: "이더넷 회사망".into(),
            identity: Identity {
                luid,
                guid: [guid; 16],
            },
            up,
            received,
            sent: received,
            at,
        }
    }
    #[test]
    fn same_alias_new_identity_cannot_cross_rates() {
        let t = Instant::now();
        for change in [
            row(t + Duration::from_secs(1), 2, 1, true, 1000),
            row(t + Duration::from_secs(1), 1, 2, true, 1000),
        ] {
            let mut c = Counter::default();
            let before = c.accept(row(t, 1, 1, true, 100)).unwrap();
            assert!(c.accept(change).is_none());
            let next = c.accept(row(t + Duration::from_secs(2), 2, 2, true, 2000));
            assert!(next.is_none());
            assert_eq!(before.interface, "이더넷 회사망");
        }
    }
    #[test]
    fn down_reset_stale_and_time_regression_break_baseline() {
        let t = Instant::now();
        for change in [
            row(t + Duration::from_secs(1), 1, 1, false, 200),
            row(t + Duration::from_secs(1), 1, 1, true, 1),
            row(t + Duration::from_secs(4), 1, 1, true, 200),
            row(t, 1, 1, true, 200),
        ] {
            let mut c = Counter::default();
            assert!(c.accept(row(t, 1, 1, true, 100)).is_some());
            assert!(c.accept(change).is_none());
        }
    }
    #[test]
    fn exact_unicode_alias_and_utf16_limits() {
        assert!(valid_alias("이더넷 회사망"));
        assert!(valid_alias(&"😀".repeat(128)));
        assert!(!valid_alias(&"😀".repeat(129)));
        assert!(!valid_alias("bad\0alias"));
        assert!(!valid_alias(""));
    }
    #[test]
    fn healthy_sample_keeps_public_rate_contract() {
        let t = Instant::now();
        let mut c = Counter::default();
        let a = c.accept(row(t, 1, 1, true, 100)).unwrap();
        let b = c
            .accept(row(t + Duration::from_secs(1), 1, 1, true, 140))
            .unwrap();
        assert_eq!(b.rate_since(&a).unwrap().received_bytes_per_second, 40.0);
        c.reset();
        assert!(c.previous.is_none() && c.binding.is_none());
    }
}
#[cfg(test)]
mod native_layout_policy_tests {
    use super::*;
    #[test]
    fn malformed_native_alias_cannot_enter_worker_state() {
        let mut row = MIB_IF_ROW2::default();
        row.Alias.fill(65);
        assert!(alias(&row).is_err());
        row.Alias.fill(0);
        row.Alias[0] = 0xd800;
        assert!(alias(&row).is_err());
        row.Alias[0] = 10;
        assert!(alias(&row).is_err());
    }
    #[test]
    fn invalid_selection_clears_cached_epoch_before_any_native_call() {
        let mut observer = Counter::default();
        let row = Observed {
            alias: "owned".into(),
            identity: Identity {
                luid: 1,
                guid: [1; 16],
            },
            up: true,
            received: 10,
            sent: 10,
            at: Instant::now(),
        };
        observer.binding = Some(row.clone());
        observer.previous = Some(row);
        assert_eq!(
            observer.sample("invalid\0alias").unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(observer.binding.is_none() && observer.previous.is_none());
    }
}

pub(super) fn aliases() -> io::Result<Vec<String>> {
    let mut raw = std::ptr::null_mut();
    // SAFETY: valid output pointer; success allocation is held by Table.
    let code = unsafe { GetIfTable2(&mut raw) };
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    if raw.is_null() {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let table = Table(raw);
    // SAFETY: API initialized table header.
    let count = unsafe { (*table.0).NumEntries } as usize;
    if count > MAX_ROWS {
        return Err(io::ErrorKind::InvalidData.into());
    }
    // SAFETY: native field offset and native row stride preserve documented padding.
    let first = unsafe { std::ptr::addr_of!((*table.0).Table).cast::<MIB_IF_ROW2>() };
    let mut result = Vec::with_capacity(count);
    for index in 0..count {
        // SAFETY: count was bounded before traversal.
        let row = unsafe { &*first.add(index) };
        // The loopback is not useful for this observation UI. VPN/tunnels stay eligible.
        if row.Type == 24 {
            continue;
        }
        let name = alias(row)?;
        if result.contains(&name) {
            return Err(io::ErrorKind::InvalidData.into());
        }
        result.push(name);
    }
    result.sort();
    Ok(result)
}

pub(super) fn automatic_alias() -> io::Result<Option<String>> {
    use windows_sys::Win32::{
        NetworkManagement::IpHelper::{
            GetIpForwardTable2, GetIpInterfaceEntry, MIB_IPFORWARD_ROW2, MIB_IPFORWARD_TABLE2,
            MIB_IPINTERFACE_ROW,
        },
        Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC},
    };
    struct Routes(*mut MIB_IPFORWARD_TABLE2);
    impl Drop for Routes {
        fn drop(&mut self) {
            // SAFETY: exactly one owner of successful native allocation.
            unsafe { FreeMibTable(self.0.cast()) };
        }
    }
    let mut raw = std::ptr::null_mut();
    // SAFETY: valid output; AF_UNSPEC requests both local address families.
    let code = unsafe { GetIpForwardTable2(AF_UNSPEC, &mut raw) };
    if code == 1168 {
        return Ok(None);
    } // ERROR_NOT_FOUND: no routes.
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    if raw.is_null() {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let table = Routes(raw);
    // SAFETY: successful initialized table.
    let count = unsafe { (*table.0).NumEntries } as usize;
    if count > 4096 {
        return Err(io::ErrorKind::InvalidData.into());
    }
    // SAFETY: native field offset and row size account for alignment.
    let first = unsafe { std::ptr::addr_of!((*table.0).Table).cast::<MIB_IPFORWARD_ROW2>() };
    let mut best: Option<((u8, u64, u64), String)> = None;
    type CachedInterface = ((u16, u64), Option<(u32, String)>);
    let mut queried: Vec<CachedInterface> = Vec::new(); // at most64 pairs, including down interfaces.
    for index in 0..count {
        // SAFETY: validated count bounds traversal of initialized native rows.
        let route = unsafe { &*first.add(index) };
        if route.DestinationPrefix.PrefixLength != 0 || route.Loopback || route.ValidLifetime == 0 {
            continue;
        }
        // SAFETY: family discriminates the SOCKADDR_INET union.
        let family = unsafe { route.DestinationPrefix.Prefix.si_family };
        let (rank, zero_prefix) = if family == AF_INET {
            // SAFETY: AF_INET selects initialized IPv4 arm.
            (0, unsafe {
                route.DestinationPrefix.Prefix.Ipv4.sin_addr.S_un.S_addr == 0
            })
        } else if family == AF_INET6 {
            // SAFETY: AF_INET6 selects initialized IPv6 arm.
            (1, unsafe {
                route.DestinationPrefix.Prefix.Ipv6.sin6_addr.u.Byte == [0; 16]
            })
        } else {
            continue;
        };
        if !zero_prefix {
            continue;
        }
        // SAFETY: scalar Value arm of initialized NET_LUID.
        let luid = unsafe { route.InterfaceLuid.Value };
        if luid == 0 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let query_key = (family, luid);
        let cached = queried.iter().find(|(key, _)| *key == query_key);
        let candidate = if let Some((_, cached)) = cached {
            cached.clone()
        } else {
            if queried.len() >= 64 {
                return Err(io::ErrorKind::InvalidData.into());
            }
            let mut ip = MIB_IPINTERFACE_ROW {
                Family: family,
                InterfaceLuid: NET_LUID_LH { Value: luid },
                ..Default::default()
            };
            // SAFETY: initialized family and LUID selector; valid row buffer.
            let code = unsafe { GetIpInterfaceEntry(&mut ip) };
            if code != 0 {
                return Err(io::Error::from_raw_os_error(code as i32));
            }
            let candidate = if !ip.Connected || ip.DisableDefaultRoutes {
                None
            } else {
                let mut physical = MIB_IF_ROW2 {
                    InterfaceLuid: NET_LUID_LH { Value: luid },
                    ..Default::default()
                };
                // SAFETY: initialized LUID selector; valid row buffer.
                let code = unsafe { GetIfEntry2(&mut physical) };
                if code != 0 {
                    return Err(io::Error::from_raw_os_error(code as i32));
                }
                let current = observed(&physical)?;
                if !current.up || physical.Type == 24 {
                    None
                } else {
                    Some((ip.Metric, current.alias))
                }
            };
            queried.push((query_key, candidate.clone()));
            candidate
        };
        let Some((interface_metric, name)) = candidate else {
            continue;
        };
        // Product policy: prefer IPv4, use IPv6 when no eligible IPv4 default exists.
        // Only compare summed metrics inside one family; u64 prevents overflow.
        let key = route_preference(rank, route.Metric, interface_metric, luid);
        if best.as_ref().is_none_or(|(old, _)| key < *old) {
            best = Some((key, name));
        }
    }
    Ok(best.map(|(_, name)| name))
}

fn route_preference(
    family_rank: u8,
    route_metric: u32,
    interface_metric: u32,
    luid: u64,
) -> (u8, u64, u64) {
    (
        family_rank,
        u64::from(route_metric) + u64::from(interface_metric),
        luid,
    )
}
#[cfg(test)]
mod automatic_policy_tests {
    use super::*;
    #[test]
    fn preference_accounts_for_interface_metric_and_avoids_overflow() {
        assert!(route_preference(0, 20, 5, 1) < route_preference(0, 1, 30, 2));
        assert_eq!(route_preference(0, u32::MAX, u32::MAX, 1).1, 8589934590);
    }
    #[test]
    fn product_family_preference_and_stable_tie_break_are_explicit() {
        assert!(route_preference(0, 100, 100, 2) < route_preference(1, 1, 1, 1));
        assert!(route_preference(0, 10, 10, 1) < route_preference(0, 10, 10, 2));
    }
}
