# S03: 숨김·복귀 통합 재확인

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: 주 담당; 본 문서, S03-background-ui.md, README.md, docs/ARCHITECTURE.md, ADR-0002, 전용 통합 검증 기록. 기존 진행 중 S00 후보 시험은 diagnostic_cache 전용 문서·harness 담당이며 공유 제품 파일 수정 없음.
- 선행 작업: S03-background-ui, Linux/Windows activation backend, 기존 소스 통합

## 사용자 결과

작업 중 창 닫기에서 돌아가기·중단 후 종료·백그라운드 계속을 구분한다. 숨김 이후 작업 완료는 창을 강제로 띄우지 않는다. 같은 앱의 명시 재실행으로 기존 창에 복귀한다.

## 계약과 범위

최신 통합 소스의 공통 검증과 Linux Xvfb 동작을 재확인한다. Windows active GUI 실패 증거를 유지하며 사용자 화면·전역 입력 fallback을 쓰지 않는다. 정상 Git 저장소가 없으므로 기존 .git을 보존한다. 전체 P0와 UI stack 최종 채택은 이 작업의 완료 범위가 아니다.

## 인수 조건

fmt/clippy/tests/release 및 문서 검사 통과. 최신 release의 숨김/동일 worker 유지/명시 재실행 복귀를 격리 화면에서 확인하고 hash와 결과를 남긴다. 미검증 OS를 완료로 표시하지 않는다.

## 검증과 handoff

최신 통합 fmt/clippy/workspace tests/release 모두 exit0. workspace 187 passed/5 ignored. 문서 검사 138개 통과(본 결과 갱신 전). 최신 Linux release SHA256 `7e5fe81444a31806189e3a84d51d1ccf809f48bd3c83df1396c9d2fbabacd469`의 격리 GUI 재시험 exit0/36.462초: 숨김0→복귀2, 동일 app/worker identity·job/attempt 유지, heartbeat 증가, Downloaded 이후 자동 창 표시 없음, secondary exit0, owned GUI/worker/temp 정리 확인. [새 증거](../validation/background-ui/integration-refresh-background.json). 이전 실증은 [통합 기록](../validation/background-processing-integration.md), Windows 한계는 [격리 GUI 기록](../validation/windows-background-gui.md)을 따른다.

## Windows 후속 통합 확인

최종 일반 실행 snapshot `98c48919e79f713c0ecca74882ad4f4d5ad34beec19b0a2778e9f839e3f15ff7`의 [원시 결과](../validation/evidence/windows-background-gui/active-normal-final.json)를 주 담당이 직접 대조했다. 숨김·작업 유지·복귀, 동일 worker 시작 시간과 owned Job 소속, heartbeat 5→11, primary/secondary exit0, 정리 후 worker 종료 및 사용자 input desktop 불변이 모두 기록되어 있다. 초기 실패는 [검증 기록](../validation/windows-background-gui.md)에 보존한다. README와 architecture의 과거 진행 중 표현을 갱신했다. 새 접근성 후보 binary 및 전체 P0 완료를 뜻하지 않는다.

최종 범위 정리 후 주 담당 재검증: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo build --workspace --release --locked` 모두 exit0. workspace 191 passed/0 failed/5 ignored. `python3 scripts/check_docs.py` 158개 통과. 이번 재실행은 공통 코드 검사이며 GUI 증거는 위에 명시한 기존 snapshot 범위다. 정상 Git 저장소 부재로 commit은 수행하지 않았다.

## 최신 요청 범위 확인 (2026-10-03)

사용자의 “통합대기랑 백그라운드 숨김 복귀 UI” 요청은 본 작업 범위로 한정한다. 주 담당은 Linux `integration-refresh-background.json`과 Windows `active-normal-final.json`을 다시 직접 읽어 숨김·동일 worker 유지·명시 재실행 복귀·소유 프로세스 정리 및 사용자 desktop 불변을 확인했다. Linux 완료 후 자동 표시 없음도 확인했다. 기존 snapshot 증거이며 새 CPU 후보나 현재 모든 소스의 GUI 검증으로 확대하지 않는다. 추가 CPU GUI 시험은 시작하지 않고, 기존 장시간 시험은 별도 작업으로 유지한다. 이 범위는 완료이며 macOS·로그아웃·절전·전체 P0는 남아 있다.
