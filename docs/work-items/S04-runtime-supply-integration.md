# S04: Windows runtime 공급 통합 조정

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08, R11
- 담당/수정 범위: root; 본 문서, 후속 공급 연결의 crates/app/src/managed.rs 및 accessible_tools.rs (공유 변경은 root만)
- 선행 작업: Windows 공식 고정 archive/source/notice 실제 자료, 원 managed reader 수명 및 시작 복구

## 사용자 결과

Windows에서 검증된 관리 Deno/FFmpeg를 설치·선택하고 실제 다운로드의 version probe/metadata/download에 동일 구성요소를 전달한다.

## 계약과 범위

Deno 자료·수집기는 diagnostic_cache, FFmpeg source mapping 자료·수집기는 metrics_recovery, 공개 runtime 다운로드 fixture는 soak_recovery_audit가 맡는다. root는 supply package/공유 인터페이스·앱 설치/선택 통합과 전체 검증을 맡는다. 같은 파일을 동시에 수정하지 않는다. 정상 Git가 없으므로 기존 .git을 보존하고 분리 담당 파일로 진행한다. Windows만 첫 릴리스 인수하며 외부 runtime 경로는 유지한다.

## 인수 조건

공식 archive/component SHA·exact source/revision·notice mapping을 확인하고 공급 채택 이유와 크기/실행 비용을 기록한다. 선택/삭제/active 보호/취소/실패가 원본을 보존해야 한다. runtime version probe가 동일 managed guardian을 사용하고 metadata/download argv 및 Attempt 증거가 일치해야 한다. 실제 공급과 합성 version fixture의 증거를 구분하고 Windows fmt/clippy/test/release 및 실제 설치/사용을 통합 검증한다.

## 검증과 handoff

현재 runtime 설치는 supply pending으로 disabled이다. 기존 원 ZIP 설치/선택 성공을 source/notice 완비나 전체 P0 완료로 표현하지 않는다. 사용자 추가 질문 없이 공식 자료 수집 및 개발을 진행한다. 공개 배포·외부 계정·서명 결제는 이 작업에서 실행하지 않는다.


## 이번 통합 결과

[합성 public runtime 성공·오류 인수](S04-windows-managed-runtime-download-native.md)와 최신 Windows 전체 네 명령을 root 독립 검증했다. [Deno 수집](S04-windows-deno-supply-adoption.md)은 official ZIP이 exe-only여서 별도 notice 공급 계약이 필요하고 전이 source/notice 검증이 남는다. [FFmpeg Windows source mapping](S04-windows-ffmpeg-supply-adoption.md)은 두 고정 Windows 의존성 전체 source와 기존 notice73 SHA를 보완했지만 전체 source/provenance/notice bundle은 미완료다. 제품 runtime 설치 disabled 상태를 근거 없이 해제하지 않았다. 후속은 고정 multi-source package 원자적 검증/게시와 공식 runtime 실제 기능·GUI 활성화 인수다.
