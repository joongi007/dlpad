# S00: 네이티브 구현과 통합 검증

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R06, R08, R10, R11
- 담당/수정 범위: 주 담당; workspace/lock/toolchain, crates/app, CI, ADR, 통합 문서
- 선행 작업: 필수 설계 문서 읽기 완료

## 사용자 결과

네이티브 입력·테마·목록을 실행하고 렌더러 비용을 측정한다.

## 계약과 범위

Cargo/schema는 주 담당이 조정한다. 독립 core 정책·진단은 각각 하위 담당이 별도 파일로 준비한다. 빈 .git을 보존한다. 정상 저장소가 없어 branch/worktree는 만들 수 없다. 중단 후 임시 도구 소실에 대비해 Rust 개발 도구는 프로젝트 전용 .local-tools에 설치한다. 배포 의존성으로 포함하지 않는다. 사용자 Documents 대신 합성 fixture를 쓴다.

## 인수 조건

원래 S00의 renderer/IME/접근성/3 OS 게이트를 유지한다. 실행할 수 없는 검증은 완료로 쓰지 않는다. 10,000행 bounded viewport 화면을 제공한다.

## 검증과 handoff

WSL2 Linux x86_64, GUI 연결 있음. Rust 1.94.0 설치 성공. git status 실패: .git은 빈 디렉터리. 프로젝트 경로의 대소문자 불일치로 기본 쓰기가 실패하여 승인된 프로젝트 쓰기를 escalated 도구로 실행한다.
