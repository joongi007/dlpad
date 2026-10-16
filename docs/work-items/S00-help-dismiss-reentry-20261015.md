# S00: Escape 후 의도 없는 자동 도움말 재개방

- 상태: implemented / Windows whole30·실제 Escape 회귀 통과, 작은 창 배치 후속
- 분류: bug
- 우선순위: P0
- 요구사항: R08 및 UI/UX 추가 인수
- 담당/수정 범위: field_reveal B/help-dismiss-reentry-v1 scratch의 ui_components 후보/회귀; Root 실행·통합·전체검증
- 선행 작업: 실제 protected native v2 200% 재개방 FAIL 보존, 현재 source/3PE29 동결

## 사용자 결과

Escape로 닫은 설명은 같은 logical focus/window 복귀/움직이지 않은 pointer 아래의 popup 제거로 다시 열리지 않는다. 명시 클릭·실제 새focus·새hover진입으로는 다시 읽을 수 있다.

## 계약과 범위

수정 후보는 ui_components.rs의 HelpDismiss/help_after_dismiss뿐이며 placement/할당/아이콘/문자열/source caller/schema는 유지한다. Window logical focus와 Escape return의 gained 신호, stationary pointer의 layer hover 복귀를 실제 Context3시험으로 분리한다. focus-only와full pointer 후보를 Root 실제 결과로 선택한다. 고정owner별 상한record와 data-lock 밖 Context 조회를 유지한다.

## 인수 조건

RawInput focus roundtrip에서 원Memory owner 불변·응답focus변화·본문닫힘; 실제다른 TextEdit focus에서 info pin/Escape복귀 후 닫힘; 실제popup이 가린 stationary owner에서 hover복귀하지만 닫힘. 기존 새focus/hover/click재허용·disabled·outsideclick·placement·field 회귀 모두 통과해야 한다. setup 전제를 실패한 시험을 재현으로 기록하지 않는다. Windows 새whole/currentPE native로 원200% 실패를 다시 검증한다.

## 검증과 handoff

원 native sample4 body재개방과 실패PNG는 사실이며 원인은 아직 미확정이다. rustfmt0만 확인, Cargo/native 미실행. Root가 독립3RED→최소GREEN→기존회귀→strict/full→native 순으로 실제 증거를 확보한다. 사용자/데스크톱에 글로벌 입력이나 fallback을 추가하지 않는다.

Root가3가설과시험setup/semantic구분을검토해S전용RED부터실행한다. 최초stage helper가before파일명조회에서실패했으나파일수정전이었고현재실제파일명으로base SHA를대조했다. 제품소스29/3PE는그대로이며새캐시/의존성/스키마는도입하지않는다.

Root actual RED compile 성공, 1 PASS/2 FAIL이다. window focus false→true에서Memory owner가같고responsefocus만바뀌며본문이재개방됐다. stationary pointer에서popup제거로hoverfalse→true가된후본문이재개방됐다. Escape owner반환/gained 가설은현재조건에서PASS한음성근거다. 두실패가각각관측돼focus-only로는부족하므로logicalfocus+실제pointer region재진입후보를선택한다. native200%의직접원인은새GUI재검증전확정하지않는다. 제품Live29/3PE는그대로두고S에서fullGREEN을검증한다.

Root fullGREEN 실제3 PASS/0 FAIL 및 기존도움말11·경로2 회귀가통과했고strictworkspace/all-targetsclippy도실제0이다. 원3개선택검사/Candidate/source불변을대조하는ui_components.rs1파일채택helper를준비했다. 현재Root29를사용하는독립Chzzk진단이종료되면Source를채택하고새whole/native를확인한다. runtime/PE비용은새build전미측정이다.

## v30 실제 통합 검증

Root가 `root-help-v30-integration-v1/adopt.py`로 `crates/app/src/ui_components.rs` 한 파일을 채택했다. `windows_msvc_workspace_probe.py --configuration docs/validation/evidence/windows-msvc-ui-feedback-v29-whole-20261015/configuration.json --output docs/validation/evidence/windows-msvc-ui-feedback-v30-whole-20261015 --reuse-owned-target`가 실제 종료0이다. `root-dragdrop-integration-v1/review_whole.py`가 원 로그·현재674 source entries·lock·3실행 파일을 독립 대조했다. fmt/strict workspace all-targets clippy/test/release 모두0, 실제1,619 PASS/0 FAIL/42 ignored/43 harness다.

CPU 실행 파일10,322,944바이트로 v29보다1,024바이트 증가했다. 새 의존성·schema·Cargo.lock 변경은 없다. 현재 private GUI v3는 실행 중이며, 이 전체 검사 결과를 Windows200% Escape 재현 통과로 대신하지 않는다. [원 검토](../validation/evidence/windows-msvc-ui-feedback-v30-whole-20261015/root-review.json).

Root 실제 private GUI v3 종료0. 100/200% 각각 yt-dlp·FFmpeg·Deno 3행에서 Escape 후6관측 전부 title/body 닫힘·원focus/ID 유지, 앱 자연0·Job0·input desktop 불변이다. Root가 원47pins/current674 source/3PE 및20PNG CRC/SHA를 대조하고4캡처를 직접 검토했다. 단200% FFmpeg/Deno는 공간 부족 fallback으로 보호 입력을 가려 배치 인수는 남는다. 도움말 전체/물리hover/OS DPI/전체UI 완료를 주장하지 않는다. [실제 검토](../validation/evidence/current-protected-help-private-v3-20261015/root-review.json).
