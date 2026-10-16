# S03: Windows 같은 사용자 백그라운드 복귀

- 상태: done (Windows native backend; GUI는 별도 미검증 gate)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: library_identity — crates/engine/src/activation/windows.rs 및 전용 검증 자료
- 선행 작업: Linux activation API, InstanceGuard, Windows native private CLI 환경

## 사용자 결과

Windows에서 동일 data의 앱을 명시적으로 재실행하면 기존 background 앱에 고정 복귀 요청만 전달한다. 새 다운로드/경로/명령은 전달하지 않는다. 안전한 endpoint 준비 실패에는 창을 숨기지 않는 공유 UI 계약을 유지한다.

## 계약과 범위

bind(&Path), poll_restore()->bool, send_restore(&Path)와 ActivationError를 유지한다. 공유 activation.rs/mod/Cargo/lib/UI는 주 담당이 조정한다. Local named pipe는 canonical root/current user SID별 이름, FIRST_PIPE_INSTANCE/REJECT_REMOTE_CLIENTS, user-only protected DACL, crypto nonce, peer SID 확인, bounded request/ACK와 overlapped 취소를 사용한다. InstanceGuard는 server Drop까지 유지한다. 기존 외부 endpoint/nonce는 보존한다.

Root/token handle identity와 DACL을 검사한다. 새 pipe/nonce의 ACL을 생성 시 설정하고 기존 사용자 data ACL을 자동 변경하지 않는다. 테스트 ACL 변경은 새 synthetic fixture directory에만 적용한다. nonce는 create-only이며 matching root/file identity·bytes일 때만 정리한다.

## 인수 조건

- 실제 Windows same-user handshake/ACK/coalescing과 Drop 정리/재bind를 확인한다.
- 잘못된 nonce/고정 메시지 위반/oversize/idle/drip/ACK 없는 peer에 제한 시간을 적용한다.
- unsafe root/marker/public token/replacement/foreign endpoint를 거절하고 보존한다.
- worker/server/client process는 직접 소유 handle과 cancel/finite deadline으로 관리한다.
- native backend는 private desktop CLI에서 실행한다. GUI가 검증되지 않으면 별도 gate로 남긴다. macOS/실제 foreign SID를 완료로 주장하지 않는다.

## 검증과 handoff

Windows GNU native 12개 activation 시험과 전체 workspace 시험, fmt/clippy/release가 동일 source snapshot에서 통과했다. [검증 결과](../validation/windows-activation-validation.md), [source manifest](../validation/windows-activation-final.json)에 실제 ACK 경쟁 실패/수정, 제한 시간과 정리/외부 보존 증거를 기록했다. Windows 제품 GUI 복귀는 별도 private desktop 시험이 필요하며 이 backend 완료로 GUI gate를 닫지 않는다. 기존 정상 Git 저장소가 없어 .git을 보존한다. 추가 windows-sys binding feature3개는 주 담당이 조정했으며 새 product runtime/tool은 추가하지 않는다.
