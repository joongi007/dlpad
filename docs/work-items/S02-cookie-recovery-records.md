# S02: 이전 boot의 세션 쿠키 복사본 복구 기록

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery — cookies.rs, tests/session_cookies.rs 및 본 문서만
- 선행 작업: root 공개 API 검토, 기존 lease importer 및 defer_cleanup; root Cargo/시작 UI 연결

## 사용자 결과

Linux 앱전용 cache에서 기록으로 소유권이 확인되고 실제 kernel boot가 달라진 쿠키 복사본만 다음 시작 시 정리한다. 같은 boot, 기록 손상/누락, 소유권 변경은 보존한다. 실제 계정/쿠키/인증network는 사용하지 않는다.

## 계약과 범위

신규 공개 recover_sessions(cache)->Result<CookieRecoverySummary,CookieImportError> 및 bounded counts를 제안했다. 상한128 owner/scan, record4KiB. OS-random nonce marker와 cache/owner/cookie/record/marker nativeidentity를 함께 묶고0600 record/marker 및0700 owner를 create-new로 만든다. cookie path exposure 전에 file/record/marker/ownerdir/cache sync를 완료한다. /proc/sys/kernel/random/boot_id의 실제 validated UUID만 production에 사용한다. 공개 boot 인자는 없고 unit private helper만 합성 boot 시험을 한다.

기존 source·cookie bytes·URL을 record/DB/log에 넣지 않는다. Debug/error에 path/nonce/boot는 공개하지 않는다. symlink/type/mode/uid/nlink/token/identity/extra 변경은 제거하지 않는다. 이름 패턴/PID/mtime/unitGC만으로 제거하지 않는다. 정상 cleanup도 정확한 세 owned file을 확인한 뒤 제거하고 빈 owner만 rmdir한다. defer_cleanup는 기록을 남기고 FD를 닫는다. 기록 없는 crash partial은 보존한다. cache128 cap 사전 count는 앱 worker 직렬화 계약이며 악성/동시 다른 import에 대해 원자적128을 주장하지 않는다.

## 인수 조건

기존15 integration 회귀와 내부 trusted boot fixture로 sameboot 보존/previousboot 제거/invalid boot/token/identity/record교체/symlink/권한/hardlink/extra/source-sentinel/scan-cap/import-cap/privacy를 확인한다. 실제 reboot/Windows ACL/macOS/sameboot manager 종료receipt는 별도 미검증·후속이며 이 작업은 R04 전체 완료가 아니다.

## 검증과 handoff

공개 API/CacheFull은 root가 승인했다. 구현과 시험을 작성하고 source freeze를 인계한다. 실제 Cargo 시험 PASS는 아직 아니다. Cargo/schema/module/UI/shared source는 변경하지 않는다. root Cargo 검증 전까지 새 시험 PASS를 기록하지 않는다.

## 소스 동결 인계

Linux record4KiB/marker512B, OS getrandom32bytes nonce, validated actual kernel bootUUID, record canonicalJSON SHA256를 marker에 바인딩했다(기존 sha2/serde 재사용; 새 deps 없음). 두 파일 inode를 먼저 create-new로 확보해 record↔marker nativeidentity를 교차 기록한다. cookie와 두 기록 및 owner/cache directory sync 후 path를 노출한다. 정상 cleanup도 record/marker 내용과 모든 identity 및 정확3파일 목록을 검사하므로 extra/partial을 전체 보존한다. defer는 모든 기록FD까지 닫는다. Linux procfs의 held directory FD경로에서 bounded128+1 entry만 읽으며 실제 import cap은 모든 foreign entry도 센다. Linux 이외 Unix import는 기존 privatecopy 동작을 유지하고 recovery만Unavailable이다.

integration17개(기존15 및 실제 sameboot기록/privacy·foreigncapacity2개), internal recovery unit4개(이전boot/incomplete·tamper/실제3identity교체/unknownowner·invalidboot)와 기존 secretDebugunit을 작성했다. 실제 reboot나 같은boot 종료receipt 인수는 없다. rustfmt 성공, 문서 검사353개 통과(추가 fixture 이후 검사 재실행 예정). Cargo는 root가 source freeze 뒤 단독 실행한다. 복구 중 기록/파일 변경은 보존하며 동일UID 악성 동시race에 대한 원자적인conditionalunlink를 주장하지 않는다. 부분unlink 뒤 crash는 보수적으로 남은파일을 보존하며 자동완료를 보장하지 않는다.

## 최초 실제 scoped 검증 완료

root가 [실제 명령 결과](../validation/evidence/session-cookie-recovery-first/result.json)를 기록했다. `cargo test -p dlpad-engine --test session_cookies --locked` integration17 PASS, `cargo test -p dlpad-engine --lib --locked cookies::` unit5 PASS, CPU unsafe-cache UI gate1 PASS, download fault integration7 PASS 및 CPU workspace/all-target clippy 모두 exit0이다. 전용 importer/recovery-unit/ui-gate/download-fault/cpu-workspace-clippy 로그로 범위를 구분한다. 이번 Linux scoped 구현·합성 인수는 완료했지만 실제 reboot·같은boot manager 종료receipt·실제 쿠키 GUI·Windows/macOS는 여전히 미검증/후속이다. 다음 reader unwind 작업은 이 PASS snapshot 이후 변경이다.
