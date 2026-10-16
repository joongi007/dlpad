# S10: 보고서 저장 중 종료 대기 실제 검증

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: 주 담당 validation/evidence 새 증거와 이 문서. release_preparation은 /tmp의 분리 빌드·관측기 후보만 준비했다.
- 선행 작업: Windows readable whole1145와 기존 보고서 barrier·정제 보고서 저장 기능

## 사용자 결과

보고서 저장 중 명시적 종료 요청에서 실제 대기 안내와 조작 잠금을 확인하고, 저장 worker 해제 또는 타임아웃 뒤 종료 결과를 구분한다.

## 계약과 범위

기존 cpu,report-closewait-fixture feature의 DEBUG 시험 실행 파일을 새로운 protected target에 빌드한다. 일반 release/GNU 앱·Cargo.lock·사용자 자료를 덮어쓰지 않는다. 실제 현재 UI URL·preview bypass·활성 Start gate를 사용하며 원 event HANDLE_LIST·writer·ACK 계약을 유지한다. 전용 virtual desktop·합성 cookie fixture·소유 임시 export만 사용한다.

## 인수 조건

현재소스·exact feature·locked offline argv·도구핀·원본 보존, 새 root protectedSID/ACL/nativeID 전후, release/timeout 별도 시험. 원 app20표본 동안 worker ACK false·원파일0bytes·Stop/Exit disabled·실제 대기 PNG 확인. 해제 또는 타임아웃의 typed 결과와 원 fileID·reportSHA·자연 app0/Job0·private input 불변을 독립 검토한다.

## 검증과 handoff

/tmp/dlpad-s10-current-closewait-v2의 순수 argv 테스트4개와 구문 검사 통과. 주 담당이 별도 Windows feature 빌드를 시작했다. Native C#·빌드·두 runtime case는 아직 인수 전이다. 실제 waiting frame을 보지 않고 자연 종료나 feature 준비만으로 기능 완료를 주장하지 않는다. 추가 의존성은 없으며 DEBUG 파일의 비용·크기는 실제 빌드 뒤 기록한다. Clipboard·physical OS·전체 P0 완료와 구분한다.
