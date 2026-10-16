# S04: 관리 capture 실제 helper 인수

- 상태: in-progress
- 분류: native-integration
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: root; 신규 crates/engine/examples/managed_guardian_fixture.rs 및 내부 process/tools 연결
- 선행 작업: 관리 capture, v3 marker, 원 journal, 관리 headless helper

## 사용자 결과

실제 관리 reader의 성공·실패·취소 뒤 종료 증거를 확인하여 관리 namespace를 안전하게 정리하고 재사용한다.

## 계약과 범위

합성 executable을 원 Windows private desktop host로 실행한다. 현재 예제 executable의 headless guardian entry를 등록하고 실제 Prepared/Ack/Begin/Terminal/EOF와 native receipt를 확인한다. 부모 측 pending 또는 원 파일 교체는 일반 종료 성공으로 바꾸지 않는다. 인증 결합 및 startup 자동 회수는 별도 통합하며 아직 활성화하지 않는다.

## 인수 조건

실제 metadata, 일반 성공, 도구 exit 7, 실행 취소를 원 guardian 프로세스로 시험한다. 각 완료 후 원 namespace 정리·동일 root 다음 실행을 확인하며 외부 합성 sentinel을 보존한다. 실제 사용자 영상·계정·화면을 사용하지 않는다.

## 검증과 handoff

구현 및 실행 결과는 아래에 기록한다. 예제 시험만으로 제품 다운로드 전체 P0 완료를 주장하지 않는다.

첫 전체 native 검증은 Clippy의 items_after_test_module 규칙으로 실패했다. 추가 wrapper를 테스트 모듈 앞 제품 항목으로 이동해 수정했다. Windows-managed-capture-connected-first 실패 로그는 보존한다.

Windows 전체 fmt/clippy/test/release는 capture-item-order-retry에서 통과했다. 실제 helper 첫 시도는 host 필수 argv 누락으로 제품이 실행되지 않았다. argv 수정 후 metadata 첫 실행에서 TerminalUnconfirmed가 발생해 실제 인수는 미완료다. 원 로그와 고정 바이너리 SHA를 보존하고 합성 fixture journal 상태로 실패 단계를 확인한다.

단계 계측으로 실패를 관리 helper의 spawn 경계로 좁혔다. 공통 native spawn_independent_helper가 cookie 전용 marker만 허용해 새 관리 marker를 거절하고 있었다. 5개 원 capability 계약을 유지하며 두 정확한 내부 marker만 허용하도록 수정했다. 임시 제품 단계 출력과 fixture의 다른 HANDLE journal 읽기 계측은 제거했다. 후자는 원 journal 잠금에 의해 오류 33이 발생하여 원 오류를 가렸으며 실패 증거는 보존한다.

관리 전용 실제 helper 인수 통과: windows-managed-capture-exact-marker-retry에서 metadata JSON, 성공, 도구 실제 exit 7, 실행 취소와 원 journal/receipt 정리 후 재사용을 모두 확인했다. 주 담당이 acceptance.json, 원 로그, 현재 소스 SHA와 고정 예제 binary SHA 9e06c75cdb675774bbb458cd111abed9fd52b9b4f88aa750adb26837cd64e7c6를 독립 대조했다. 실제 관리 Job/helper 경로의 합성 인수이며 cookie 결합과 제품 DownloadSession 전체 전달 및 startup 자동 회수는 미검증이다. spawn marker 수정 후 전체 workspace 회귀는 별도 진행 중이다.

최종 spawn 수정 후 Windows GNU CPU workspace fmt·Clippy(-D warnings)·test·release 모두 PASS: windows-managed-capture-exact-marker-workspace. 주 담당이 최종 소스 SHA와 현재 파일을 독립 대조했다. 임시 단계 출력은 제품 소스에서 제거됐다. 제품 전체 P0 및 인증 결합/startup 회수 완료는 아니다.
