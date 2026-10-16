# S04: Windows 시작 worker 관리 복구 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: root; crates/app/src/accessible.rs, crates/app/src/catalog.rs, 본 문서
- 선행 작업: public ToolManager::recover_execution 실제 Windows 부모 종료 인수와 원 receipt/DB 독립 검증

## 사용자 결과

Windows CPU/accessible 앱은 시작 및 기존 이전 작업 다시 확인 행동에서 원 관리 실행의 완료 기록을 worker에서 회수한다. 증거가 미완료면 원 파일을 보존하고 관리 도구 실행·변경 보류를 안내한다. 외부 도구 작업은 기존 정책대로 가능하다.

## 계약과 범위

관리 root가 이미 존재할 때만 검사하여 시작만으로 빈 managed-tools 폴더를 만들지 않는다. cookie pending과 managed pending을 별도 상태로 유지한다. 손상/권한 실패는 기존 startup failure gate를 유지하며 자동 다운로드 재시작은 하지 않는다. 이미 준비된 종료 기록만 원 API로 회수한다. 긴 I/O는 기존 단일 worker/channel1 안에서 수행하고 작업/CloseWait 중 recovery 실행을 거절한다. UI 안내는 중앙 catalog 문자열이다. 제품 GUI 시각 검증은 native caller 테스트와 구분한다.

## 인수 조건

Windows absent root는 미생성, live/dropped pristine은 pending 보존·관리 변경 거절, malformed marker는 startup failure와 원본 보존. 다시 확인과 CloseWait/다른 worker의 경합은 새 작업을 만들지 않는다. 실제 완료 회수 API 증거와 앱 호출 전달을 각각 검증하며 Windows 전체 fmt/clippy/test/release를 통과한다.

## 검증과 handoff

실제 public paired parent crash 인수 통과 후 호출 연결 시작. 아직 최신 앱 GUI 동작/전체 검증을 완료했다고 주장하지 않는다.

앱 호출 최종 검증: `windows-managed-startup-app-workspace`에서 Windows GNU CPU fmt/clippy/workspace test/release 전부 PASS, 신규 시작/보류/변조/재확인/CloseWait 호출 테스트 5개도 실제 통과했다. 주 담당 final source SHA와 현재 파일을 독립 대조했다. Windows 앱의 기존 단일 worker에 자동 회수를 연결했고 managed pending을 cookie pending과 구분하며 관리 경로의 실행/변경 차단과 외부 경로 허용 정책을 유지한다. 실제 최신 GUI 시작·클릭·시각 동작은 후속이며 전체 P0 완료가 아니다.

실제 pending GUI startup PNG를 주 담당이 읽어 관리 보류 안내가 스크롤 아래에만 있어 처음 화면에서 보이지 않는 점을 확인했다. 보류 상태는 기존 항상 보이는 status에도 같은 중앙 catalog 안내를 표시하도록 보강했다. 기존 warning 위치·외부 경로 허용 정책을 유지한다. 최신 바이너리의 실제 재검증은 진행 중이다.

최신 pending-visible 실제 GUI도 PASS. 주 담당 startup.png를 직접 읽어 첫 viewport의 status에 중앙 관리 보류 안내가 보이는 것을 확인했다. 재확인·원 파일 SHA·자연 Job 종료 및 최신 고정 binary/current source SHA도 독립 대조했다. 최신 Windows 전체 네 검증과 문서 612개 검사 PASS, 전체 P0는 미완료다.
