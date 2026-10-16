# S04: Windows 관리 reader 수명과 안전한 복구 통합

- 상태: in-progress
- 분류: safety-recovery
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: root: tools/execution_lease.rs, tools/execution_marker.rs, process 관리 guardian 연결 및 공유 선언. 하위 담당의 파일 변경은 개별 배정 후 수행한다.
- 선행 작업: Windows managed records, guardian config, native journal의 실제 Windows 컴파일·회귀 검증

## 사용자 결과

관리 도구 실행 중 앱이 종료되어도 실행 중인 reader의 파일을 변경하지 않는다. 소유 작업 종료와 원 receipt가 확인된 경우에만 다음 실행에서 관리 도구를 다시 사용할 수 있다.

## 계약과 범위

exclusive 관리 잠금 아래 원 marker·lock HANDLE과 journal·8 receipt를 생성하고 최종 immutable 원 식별 정보를 동기화한다. 기존 v2 marker와 부분 기록은 자동 회수 대상으로 승격하지 않는다. guardian에는 원 receipt HANDLE을 명시 allowlist로 전달한다. Intent, 실제 suspended 프로세스 identity, Bound 동기화, 원 journal 잠금 handoff, 실행 허용 순서를 유지한다. Job 전체 종료 뒤에만 receipt를 작성한다. 부모는 Terminal 및 EOF 뒤 잠금을 재획득하여 원 receipt를 대조한다.

정상 종료와 startup 회수는 모든 원 DELETE HANDLE의 경로·identity·ACL·receipt를 사전 검증한다. journal 및 receipt 정리 후 marker를 마지막에 삭제한다. 사용자 설치·외부 파일·알 수 없는 기록은 보존한다. 공유 schema 변경과 통합 검증은 root가 담당한다.

## 인수 조건

실제 Windows 합성 fixture로 정상 종료, 실패, 취소, 앱 원 프로세스 강제 종료 후 fresh startup, 살아 있는 reader 보존, 잘못된 receipt·ACL·파일 교체 보존, 다음 관리 실행 재사용을 검증한다. 단위 테스트나 합성 receipt 쓰기만으로 실제 Job 종료 권한을 주장하지 않는다. 사용자 화면·실제 영상·실제 인증은 사용하지 않는다.

## 검증과 handoff

읽기 전용 병렬 감사가 현재 marker-only finish, 원 HANDLE 접근 부재, v2 anchor 부족, journal 잔여물 문제를 확인했다. 구현 및 실제 native 실행 증거는 후속으로 기록한다. 이 작업은 아직 완료되지 않았다.

관리 guardian 제어 프레임은 별도 magic DLPMG001로 분리하고 8개 reader 슬롯을 허용한다. 기존 cookie 프레임 DLPWG001의 2개 슬롯 계약은 유지한다. 슬롯 경계·cookie 도메인 혼동 거절을 별도 회귀 검증한다. 신규 담당 파일: process/windows_managed_guardian_protocol.rs.

제어 프로토콜 실제 실행: `cargo test -p dlpad-engine --lib process::windows_managed_guardian_protocol --locked`에서 5 passed / 0 failed. 슬롯 0..7 왕복, 슬롯 8 거절, cookie magic 거절, 모든 reserved byte/잘못된 길이/역할별 payload 거절을 확인했다. 이는 공통 순수 프레임 검증이며 새 helper의 실제 Windows Job 종료 검증은 아니다.

Windows begin_execution에 exclusive 잠금 아래 journal 생성과 v3 immutable marker anchors/digest 기록을 연결했다. 정상 finish는 원 live pristine journal의 실제 삭제를 확인한 후 marker를 마지막에 정리한다. 아직 reader가 journal에 연결되지 않았으므로 cleanup_completed 및 startup 자동 회수는 후속 구현 대상이다. 중간 생성/쓰기 실패와 기존 v2 pending은 보존한다. 새로운 v3 schema는 주 담당이 조정했으며 실제 Windows 전체 시험 전이다.

부모 HANDLE allowlist 런처는 공통 bounded 송신을 재사용하되 관리 전용 headless marker를 사용한다. 신규 관리 entry는 GUI/DB 초기화 전에 구분하고 inherited capability 5개의 type/unique/상속 플래그를 검사한다. 관리 runtime과 삭제권한 없는 독립 journal facade를 공유 선언에 연결했다. 원 Job 종료 후 receipt 생성 구현은 실제 native 검증 대기이며 부모 실행 adapter 연결은 여전히 남는다.

첫 native 통합 Clippy는 TransferLock wrapper를 File로 잘못 전달한 오류로 실패했다. 기존 wrapper의 원 File 접근을 Windows 내부 API로 명시하여 journal 원 잠금 pin과 checked unlock을 연결했다. 수정 파일 direct_download.rs가 root 통합 범위에 추가된다. 첫 실패 로그는 windows-managed-lifecycle-native-first에 보존한다.

### 실제 Windows 통합 검증

주 담당이 `python3 scripts/windows_current_workspace_probe.py .../windows-managed-lifecycle-lock-capability-retry --execute --seconds 1800`을 실행했다. fmt, 전체 workspace Clippy(-D warnings), workspace test, release build가 모두 통과했다. result.json 및 최종 소스 SHA를 현재 파일과 독립 대조했다. native journal 7개, 삭제권한 없는 guardian journal 3개, runtime 제어 pipe 2개 및 v3 실제 정리 후 재사용 시험이 통과했다. 사용자 input desktop은 그대로였다. 문서 검사는 587개 PASS다. 실제 관리 도구 실행 adapter 및 startup 자동 회수는 아직 연결되지 않았으며 이 시험을 실제 다운로드/앱 crash 전체 인수로 확대하지 않는다.

관리 실행에 worker용 run_tool capability를 추가한다. 원 v3 marker와 mutable journal의 anchors/nonce/baseDigest를 대조하고, baseDigest도 canonical v2 원 바인딩에서 재계산한다. 각 stage를 8개 single-use 슬롯으로 전달한다. 정상 finish는 pristine 또는 실제 모든 receipt 확인이 있는 completed 정리를 구분한다. 인증 결합과 DownloadSession/Deno discovery 전달은 남아 있어 startup 자동 회수는 아직 활성화하지 않는다.

관리 전용 실제 helper 인수 통과: windows-managed-capture-exact-marker-retry에서 metadata JSON, 성공, 도구 실제 exit 7, 실행 취소와 원 journal/receipt 정리 후 재사용을 모두 확인했다. 주 담당이 acceptance.json, 원 로그, 현재 소스 SHA와 고정 예제 binary SHA 9e06c75cdb675774bbb458cd111abed9fd52b9b4f88aa750adb26837cd64e7c6를 독립 대조했다. 실제 관리 Job/helper 경로의 합성 인수이며 cookie 결합과 제품 DownloadSession 전체 전달 및 startup 자동 회수는 미검증이다. spawn marker 수정 후 전체 workspace 회귀는 별도 진행 중이다.

최종 spawn 수정 후 Windows GNU CPU workspace fmt·Clippy(-D warnings)·test·release 모두 PASS: windows-managed-capture-exact-marker-workspace. 주 담당이 최종 소스 SHA와 현재 파일을 독립 대조했다. 임시 단계 출력은 제품 소스에서 제거됐다. 제품 전체 P0 및 인증 결합/startup 회수 완료는 아니다.
