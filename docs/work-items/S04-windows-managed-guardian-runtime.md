# S04: Windows managed guardian runtime

- 상태: implemented; integration-validation-pending
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 신규 process/windows_managed_guardian.rs와 본 문서만
- 선행 작업: managed config/record, metrics_recovery managed guardian journal, root native helper/module integration

## 사용자 결과

관리 도구는 durable Intent/Bound와 독립 helper journal 검증 뒤에만 실행하고 원 Job terminal 확인 뒤 원 receipt를 기록한다.

## 계약과 범위

cookie 타입을 재사용하지 않는다. native PreparedChild/Job/128byte Frame 및 bounded control polling 계약을 사용한다. actual creation→Prepared→app Ack→독립 acquire→helper Ack→Begin→Resume→Job empty→receipt→Terminal. root_path 문자열/프레임 decode 자체는 native 소유권 권한이 아니다. 기존 module/Cargo/source 변경 제외.

## 인수 조건

EOF/malformed/replay/partial/timeout는 원 소유 Job을 정리하며 native terminal 확인 실패 시 receipt/Terminal을 발신하지 않는다. 원 stdout/stderr writer는 prepare 직후 drop한다. 실제 native tests와 parent wiring은 root 담당이며 코드 작성만으로 완료 주장하지 않는다.

## 검증과 handoff

기존 cookie Frame slot>1 거절을 발견해 root에 공유 계약 조정을 요청했다. root는 별도 `windows_managed_guardian_protocol`(magic DLPMG001/index0..7/동일 Frame API)을 담당한다. 이 신규 모듈만 import하여 cookie protocol/타입을 변경하지 않는다.

metrics_recovery와 `ManagedGuardianJournal::acquire(root_path,intent,index,nonce,creation,receipt)->io::Result<Self>` 및 `complete(&mut self)->io::Result<()>` 계약 합의. runtime은 `crate::tools::ManagedGuardianJournal` 재노출을 사용하므로 root 연결이 필요하다. 신규 source SHA256 `c7f5d3119a1b877714f202100e11e6fefe925c9e45fce37290acad61aebdef2c` 동결. index/nonce 및 이미 등록된 Intent를 native prepare 전에 검사한다. 이후 suspended child 실제 creation/expected Bound digest와 양방향 Ack/Begin을 검증하고 control EOF/partial/replay를 failclosed 처리한다. gate 실패는 cancel_status→원 Job 확인→acquired journal receipt→실제 exitcode Terminal 순서; acquire 전 실패는 receipt 없이 원 prepared child 취소. Resume 경계 Interrupted는 실제 status가 없어 receipt/Terminal을 만들지 않는다.

관리 index7을 사용한 실제 anonymous control pipe partial128/완성/EOF 및 partial EOF/elapsed deadline native 회귀 두 개를 작성했다. 아직 실행하지 않았다. Cargo/rustfmt/native 실행/parent module/기존 guardian 수정 없음. root의 별도 protocol·journal·helper entry 연결, Windows 컴파일과 실제 process/Job/receipt/EOF 인수 검증이 남았다. **코드 작성만 완료, 실제 runtime 검증 미완료**다.

### 실제 Windows 통합 검증

주 담당이 `python3 scripts/windows_current_workspace_probe.py .../windows-managed-lifecycle-lock-capability-retry --execute --seconds 1800`을 실행했다. fmt, 전체 workspace Clippy(-D warnings), workspace test, release build가 모두 통과했다. result.json 및 최종 소스 SHA를 현재 파일과 독립 대조했다. native journal 7개, 삭제권한 없는 guardian journal 3개, runtime 제어 pipe 2개 및 v3 실제 정리 후 재사용 시험이 통과했다. 사용자 input desktop은 그대로였다. 문서 검사는 587개 PASS다. 실제 관리 도구 실행 adapter 및 startup 자동 회수는 아직 연결되지 않았으며 이 시험을 실제 다운로드/앱 crash 전체 인수로 확대하지 않는다.
