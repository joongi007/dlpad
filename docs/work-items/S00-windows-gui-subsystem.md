# S00: Windows CPU 후보의 GUI subsystem

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R01, R08, R11
- 담당/수정 범위: root; crates/app/src/accessible_cpu_main.rs 및 본 문서
- 선행 작업: windows-idle-job-members-corrected-20261004T132709Z

## 사용자 결과

Windows CPU GUI 실행에서 콘솔 host를 별도로 생성하지 않는다.

## 계약과 범위

Rust Windows GUI subsystem 속성만 CPU entrypoint에 추가한다. 별도 라이브러리·feature·dependency를 추가하지 않는다. 엔진 guardian의 전달된 pipe/handle 계약은 유지하고 실제 인증 실행 회귀로 확인한다. 다른 OS UI 작업을 추가하지 않는다.

## 인수 조건

Windows fmt/Clippy/test/release, 격리 GUI 시작·자연 종료 및 Job 구성원 관찰, helper/authenticated execution 회귀를 실행한다. subsystem 변경만으로 메모리·CPU 예산을 통과했다고 주장하지 않는다.

## 검증과 handoff

원 native Job 관측은 세 회 모두 app 직접 자식 C:/Windows/System32/conhost.exe를 확인했다. 전체 collector는 foreign image 정책에서 실패했고 비용 표본은 없다. CPU entrypoint에 Windows subsystem 속성이 없음을 소스에서 확인했다. 실제 빌드·GUI·helper 검증은 후속이다. 콘솔 stderr는 대화형 터미널에 자동 표시되지 않을 수 있으므로 전달된 diagnostic pipe와 GUI 오류 상태를 검증한다.

후속 실제 검증: `windows-cpu-gui-subsystem-workspace-20261004T132856Z`의 fmt/Clippy/test/release 모두 성공했고 root는 현재 source 및 CPU release SHA를 직접 대조했다. `windows-idle-gui-subsystem-20261004T133036Z`의3회 Job 관측은 원 app만 확인하고 WM_CLOSE exit0·자연 Job 비움·private desktop 닫힘을 기록했다. conhost 제거는 실제 관측됐다. CPU0.311/0.466/0.932%, sampled WS85,266,432/79,921,152/79,515,648bytes로 첫 회80MiB 예산을 초과해 전체 collector는 FAIL이다. 사용자 host quiet 상태는 확인하지 않았고 이 결과로 전체 성능/stack 인수 완료를 주장하지 않는다. helper/authenticated pipe 회귀는 후속이다.
