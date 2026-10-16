# S03: Windows suspended spawn과 Job 소유권

- 상태: done
- 분류: reliability
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: library_identity — crates/engine/src/process.rs, process/windows.rs, 전용 검증 자료
- 선행 작업: S00 Windows GNU native CLI 검증 환경

## 사용자 결과

다운로드 도구가 시작 직후 자손을 생성하더라도 취소·시간 초과·앱 비정상 종료 시 앱이 소유한 Windows 프로세스 트리만 종료한다.

## 계약과 범위

std Child는 CreateProcess의 primary thread handle을 노출하지 않는다. Windows 전용 native adapter가 stdio allowlist와 JOB_LIST attribute로 생성 자체에 Job 소유권을 적용하고 suspended process membership을 확인한 뒤에만 주 스레드를 재개한다. Job handle과 pipe reader는 자식에 상속하지 않는다. argv quoting은 Windows 규칙에 따르며 shell을 실행하지 않는다. Cargo feature 변경은 통합 담당자가 조정한다. 기존 Linux guardian 동작을 유지한다. MSVC 설치와 사용자 데스크톱 실행은 범위 밖이다.

## 인수 조건

- Job 할당 실패 시 아직 suspended인 소유 자식만 종료하고 기다린다.
- 실제 Windows GNU에서 즉시 자손 생성, 취소, 시간 초과, 부모 crash를 검증하며 무관한 fixture 생존을 확인한다.
- Windows 명령은 숨긴 private desktop의 전용 CLI host에서 실행한다.
- Linux process/guardian 회귀, fmt/clippy 검증을 실행한다.
- Windows GNU 검증을 MSVC/macOS 완료로 표현하지 않는다.

## 검증과 handoff

[실제 Windows 검증](../validation/windows-process.md)에 source snapshot, 실행 명령/로그, 남은 OS 게이트를 기록했다. 최신 native fmt/clippy/test/release4개 모두 동일 source hash로 통과했다. Linux process12/guardian8 및 engine all-target clippy가 통과했다. 실제 Windows tree4case, literal argv, repeated failed spawn64회(handle139→139), Resume전 Job close, invalidJob 실패를 시험했다.

Windows 11 GNU에서 확인했으며 Windows 10 최소지원 제안, MSVC, macOS, GUI/IME/a11y/logout/sleep은 완료로 주장하지 않는다. Cargo feature는 주 담당자가 조정했다. product 의존성을 새로 추가하지 않았다. 정상 Git 저장소가 없어 기존 .git을 보존하고 파일 독점으로 조정했다. 중앙 ADR/P0 상태와 후속 신규 소스의 전체 통합 검증은 주 담당에게 전달했다.
