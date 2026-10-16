# S00: 최신 Windows cookie gate와 seed 전체 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 이 문서, 신규 native evidence, P0-status, 병렬 보드
- 선행 작업: 실제 late Terminal 수정 및 startup fixture3case

## 사용자 결과

현재 source의 Windows GNU CPU 전체 lint/test/release 결과를 이전 snapshot과 구분한다.

## 계약과 범위

Rust/Cargo/vendor/CLI runner source를 동결하고 기존 project-local native toolchain과 private desktop host를 사용한다. 같은 source의 Linux 전체 명령은 이미 terminal exit0/SHA일치이다.

## 인수 조건

capability/fmt/clippy/test/release 실제 exit0, 입력 source SHA 일치, input desktop 불변, bounded 새 evidence. MSVC/macOS/실제GUI 기능 전반은 이 범위가 아니다.

## 검증과 handoff

`python3 scripts/windows_current_workspace_probe.py <canonical-project>/docs/validation/evidence/windows-cookie-late-seed-native-workspace-first --capability --execute` 실제 실행 중. 터미널 결과 전 완료로 표시하지 않는다. 원 실패도 보존하고 종료 후 명령별 결과와 current SHA를 대조한다.

## 실제 terminal과 통합 검토

collector terminal exit0. capability/fmt/clippy/test/release 각 exit0·passedtrue·sourceunchangedtrue·inputdesktopunchangedtrue를 원 JSON에서 확인했다. source_final360개 파일의 SHA를 현재 worktree와 대조해 불일치0. CPU executable7452672bytes/SHA6898fd11ea7debf031efd5ce8fc8b734356ad4720ff4b0aea44ccac7dc63daea이며 실제 startup GUI3case 바이너리와 같다. 이 완료 범위는 Windows GNU CPU CLI이며 ignored native acceptance·MSVC/macOS·전체P0를 완료로 표시하지 않는다. Rust source freeze는 이 collector의 terminal/SHA 검토 이후 해제 가능하다.
