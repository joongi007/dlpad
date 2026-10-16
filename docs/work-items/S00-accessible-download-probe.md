# S00: Accessible native candidate engine GUI acceptance

- 상태: completed (bounded Linux candidate GUI vertical slice)
- 분류: verification
- 우선순위: P0
- 요구사항: R01, R04, R08, R13
- 담당/수정 범위: guardian_fault; 신규 scripts/accessible_download_probe.py 및 전용 validation/evidence
- 선행 작업: root accessible candidate build 준비 신호; 기존 DownloadRepository/guardian

## 사용자 결과

새 native 후보에서 URL 입력과 다운로드 성공/실패 이력, 정제 보고서 복사, 일시 중단/새 Attempt 재개, 활성 작업 닫기 CancelWait를 격리 GUI로 검증한다.

## 계약과 범위

README/PRODUCT/CONTEXT/관련 요구사항/ADR 및 DESIGN 계약 유지. candidate/main/Cargo 변경 금지. 기존 native/background/x11 helper는 read-only import. 사용자 display/clipboard/계정/Documents/실제영상 금지; 보고서 복사는 owned Xvfb의 독립 CLIPBOARD selection에만 한정한다. tempfile synthetic executable/output/SQLite 사용, partial64KiB/worker최대120초, app 로그64KiB상한. root 준비신호 전 후보 실행/빌드 금지. 기존8h/archive유지.

## 인수 조건

binary/source/fixture hash와ownedPIDstart ticks 증거; GUI URL→SQLite Downloaded 및 artifact bytes; Failed와 정제 report copy; Paused 보존→sameJob/newAttempt/reextract/resume; closeprompt StopExit→Cancelled journal→app/workerreap. GUI 실패는 입력/레이아웃 불확실성과 제품 원인을 분리하고 사용자 화면 fallback 금지. 키입력은 synthetic X11이며 실제IME/screenreader 증거 아님.

## 검증과 handoff

신규 script 작성/py_compile 먼저. 후보 준비 후 inspect layout→case별실행→증거 문서 갱신. 미실행은 PASS로 기록하지 않는다.

Final all exit0/4casePASS on rootfix SHA55ecdbc89c4eb42c4e898e939e7ca164e94ed5359848a903c7d58d77a452401f. 최초pause productbug/closecoordinatefailure도보존. RawSQLite integrity4PASS, report1423B정제검증, bounded evidence873013B. [검증/실제명령/한계](../validation/accessible-download-validation.md). P0전체완료/스택채택/IME/screenreader는root다른게이트.
