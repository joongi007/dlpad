# S02: 앱 cookie reader gate 연결 계약 감사

- 상태: proposed
- 분류: review
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만; 제품 코드는 읽기 전용
- 선행 작업: reader journal, C/I/B gate 및 launcher receipt FD 기반

## 사용자 결과

앱 crash 이후 같은 부팅에서 private cookie 복사본을 복구하려면 실제 reader 시작 전에 Intent 및 native identity를 디스크에 기록해야 한다. 이 문서는 root의 제품 연결 선택을 위한 계약이며 구현 완료를 뜻하지 않는다.

## 현재 코드 근거

cookies.rs의 공개 `begin_reader()`는 현재 atomic pending reservation만 시작한다. `SessionCookieReader::confirm()`은 atomic을 false로 바꾸며 journal을 변경하지 않는다. OwnedCopy의 journal은 `Option<ReaderJournal>`이고 cleanup은 journal의 `cleanup_eligible()`를 확인한다. 따라서 현재 guard confirm과 durable journal resolve는 별개다.

reader_journal.rs의 begin은 receipt를 create-new하고 Intent와 receipt 식별자를 sync한 뒤 writer 및 같은 OFD record clone을 반환한다. bind는 canonical identity digest를 검증하고 Bound를 sync한다. resolve는 실제 256바이트 receipt를 읽어 정확한 identity 전체 일치를 검증하고 Resolved를 sync한다. download.rs는 metadata 실행 직전과 download 실행 직전에 기존 guard를 만들며 확인된 terminal 반환에서 confirm한다. version 호출에는 guard가 없다.

## 최소 API 및 파일 담당

Linux 전용 내부 `begin_gated_reader(slot: ReaderSlot) -> Result<GatedCookieReader<'_>, CookieImportError>`를 추가한다. 기존 공개 `begin_reader()`와 Windows 경로는 호환성을 유지한다. ReaderSlot은 Metadata=0, Download=1 두 값이며 임의 index나 외부 nonce를 UI에서 받지 않는다. nonce와 intent token은 내부 난수, boot은 검증된 현재 부팅 식별자를 사용한다.

GatedCookieReader는 lease를 borrow하며 다음 좁은 인터페이스를 제공한다.

- `take_launch(&mut self) -> Result<ReaderLaunch, CookieImportError>`: 한 번만 capability와 writer/lock 소유권을 이관한다. begin의 durable Intent 완료 이전에는 반환하지 않는다.
- `bind_identity(&mut self, &ReaderIdentity) -> Result<(), CookieImportError>`: slot 및 capability 전 필드 일치를 확인하고 journal.bind가 파일·directory sync를 완료한 뒤 반환한다. 이 성공 후에만 process가 B를 보낸다.
- `confirm_terminal(self) -> Result<(), CookieImportError>`: 실제 receipt를 journal.resolve로 재검증·sync한 뒤에만 atomic pending을 해제한다. 결과 없는 Drop, panic 및 poisoned mutex는 pending을 보존한다.

root는 cookies.rs/OwnedCopy의 journal Mutex와 process의 optional observer API 및 download 연결을 조정한다. process는 `ProcessSpec` 필수 필드를 추가하지 않고 기존 공개 호출을 보존하며 authenticated 전용 wrapper 또는 optional observer를 사용한다. 기존 reader gate/codec 모듈의 schema 변경은 필요하지 않다.

Mutex는 begin, bind, resolve 및 검증의 짧은 journal 작업 동안만 보유한다. 실행 전체나 C/I 대기 동안 보유하지 않는다. 현재 records.validate 및 owned_names/cleanup도 동일 mutex로 정확한 journal snapshot을 읽어야 한다. poisoned 상태를 임의 복구해 삭제하지 않는다. disk sync는 짧은 작업이라는 이유로 UI thread에 옮기지 않는다.

## 실행 순서와 실패 불변식

argv와 cookie capability를 먼저 검증한다. begin은 single-reader CAS 후 durable Intent를 기록한다. Intent 기록 시도 뒤 실패는 결과적으로 어느 단계까지 남았는지 불확실할 수 있으므로 atomic을 보수적으로 유지한다. 원본 검증만 실패한 명백한 pre-Intent 경로만 reservation을 해제할 수 있다.

process가 소유 launcher를 만든 뒤 C를 보내고 I를 정확히 읽는다. callback의 bind sync 성공 뒤 B를 보내며 R까지 기다린다. cancel 및 전체 deadline은 이 단계 전부를 포함한다. C 실패, 잘린 I, 잘못된 capability, bind I/O 실패, B 쓰기 실패 또는 R 이전 취소는 tool 시작을 확인하지 않은 경로이며 pending을 해제하지 않는다. launcher writer/lock FD는 stdio 이외 allowlist로 전달하고 즉시 CLOEXEC를 복구하며 service/tool에는 전달하지 않는다.

정상 tool 반환이나 확인된 실패·취소 후에도 atomic 해제 전에 실제 durable receipt resolve가 필수다. resolve 실패는 고정 SessionCleanupDeferred로 매핑하여 복사본을 보존한다. GU/LU 또는 typed 미확인 process 결과에서는 즉시 guard confirm하지 않는다. cleanup-only receipt가 추후 저장되면 startup recovery가 실제 proof를 재검증할 수 있지만 실행 결과를 성공으로 바꾸지 않는다. 인증 로그 및 metadata free text 생략 정책은 유지한다.

## 슬롯 재사용과 인수 조건

현재 journal은 metadata0 다음 download1의 두 reader만 허용하며 Resolved 슬롯을 재사용하지 않는다. 동일 lease로 pause/resume·재시도·metadata 재호출을 하면 slot transition이 거절된다. 연결 단계에서 이를 명시적으로 처리해야 한다. 임의 슬롯 초기화는 금지한다. 각 실행 attempt마다 새로운 private lease/owner를 생성하거나, 별도 검증된 attempt별 journal 정책을 후속 설계한다. 기존 resume API가 자동으로 같은 lease를 재사용한다고 가정하지 않는다.

실제 인수는 metadata0→Bound→Resolved 후 download1→Bound→Resolved, callback 실패 시 B 및 tool spawn 0, 부분 I/B 실패 및 취소, crash 전 Intent/Intent 뒤/Bound 앞뒤/receipt sync 전후, receipt mismatch·zero·부분 파일, mutex poison 및 unwind, 반복 attempt의 명시적 거절, 원본과 foreign sentinel 보존을 포함한다. version/FFmpeg에는 cookie argv와 gate가 없어야 한다. lease borrow와 record flock은 launcher 종료 또는 보수적 defer까지 유지한다.

## 검증과 handoff

제품 코드 읽기와 계약 감사만 수행했다. Cargo/native/GUI 실행 및 제품 코드 수정은 하지 않았다. root에게 API 및 슬롯 재사용 경계를 메시지로 전달한다. 전체 제품 연결과 같은 부팅 복구는 아직 이 문서의 인수 범위로 완료되지 않았다.
