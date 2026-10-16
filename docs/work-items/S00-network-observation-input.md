# S00: 네트워크 관측 설정 입력 누락 교정

- 상태: done (scoped)
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R11, R13
- 담당/수정 범위: root crates/app/src/accessible_metrics.rs, ui_components.rs, scripts/windows_network_input_private_probe.cs 및 관측 수집기·문서·검증. 하위 담당 diagnostic_cache는 /tmp 읽기 전용 전 입력 coverage 감사. 공유 schema/Cargo/lock 변경 없음.
- 선행 작업: S00-input-caret-typography, S00-public-video-ui-walkthrough

## 사용자 결과

네트워크 관측 입력에도 다른 입력과 동일한32px 높이/12px 여백/커서 중앙/hover/focus/논리 ID와 가려진 입력 focus 노출을 적용한다. 기존 Linux 관측/Windows 측정 불가·64자 검증·빈칸 비활성화·세션 선택 정책을 유지한다. 네트워크 설정의 영구 저장은 기존에 없으며 이번 교정을 저장 기능 구현으로 보고하지 않는다.

## 계약과 범위

AGENTS 읽기 순서와 Rust native, UI thread I/O 금지, private desktop/no visible fallback을 따른다. 기존 Git은 정상 저장소가 아니므로 .git 보존. 원 TextEdit 직접 호출을 감사하되 진단 화면과 multiline의 범위 밖 수정은 후속으로 기록한다. 신규 dependency 없음.

## 인수 조건

원 네트워크 입력의 실제 native 클릭/입력/캡처로 재현; 실제 MetricsPanel.show_settings 회귀에서TextInput 높이/글자/커서 좌표와 focus/안내 변경/disabled/검증 실패를 확인한다. actual native 수정본에서빈값·유효/무효·적용·Tab/키 편집·Light/Dark 및 앱 배율을 관측한다. whole fmt/Clippy/test/release와문서검사, scoped 보고. 물리IME/OSDPI/Windows 실제망 관측 성공과 전체입력 클릭완료로 확대하지 않는다.

## 검증과 handoff

확인: accessible_metrics.rs의show_settings는직접TextEdit.singleline/min_size32만사용해공통입력의Center/margin12/focus/ID교정이빠져있다. 앞선953후보의네트워크입력전수실행근거는없었고이번작업에서명시적으로보완한다.

읽기 전용 전 입력 감사에서 네트워크와 진단 singleline3개(log index/MPD path/export path)의공통geometry누락을확인했다. 진단multiline/readonlypreview와페이지DragValue는별도규격이며전수클릭완료로보고하지않는다. 네트워크외각입력의수행근거를별도표로보존한다.

첫 native 관측의 valid 입력/PNG에서 상단 glyph와4px 여백을 root가 직접 확인했다. 중간 observer는 복수 Fill의 동일input-cookie-setter.png create-only 충돌로 FAIL했다. 원 로그와 소유 Job 정리를 보존하며 각 입력에 fresh stage를 분리해 재시험한다. 제품 적용 실패와 구별한다.

수정 전 actual MetricsPanel.show_settings 회귀는12px glyph inset assertion에서실제로FAIL했다(network-input-geometry-red cargo101). 원 native 관측 두번째는반복S10Click의동일action-before PNG 충돌로FAIL했으며관측기각action도freshstage로분리했다. 원partial valid입력과PNG/cleanup보존. 수정후actualpanel두회귀는Windows cargo exit0(정렬Light/Dark×1/1.5/2,invalidID/focus·64char·disabled)이며전체검사와새nativeGUI는진행중이다.

Windows 전체 `python3 scripts/windows_msvc_workspace_probe.py --configuration docs/validation/evidence/windows-msvc-public-ui-playback-clippy-fixed-20261008/effective-configuration.json --output docs/validation/evidence/windows-msvc-network-input-whole-20261008 --reuse-owned-target` exit0: fmt/Clippy/test/release 원exit0,41harness957passed/0failed/17ignored,source610. CPU c3bb901582bb309673c76240915db60f5704fc8f309e133cb4274bf4890273c4/9,419,776B(-512B),CLI/installer는이전원SHA와동일하다. nativeGUI현재수행중. 이전953후보의비용·공개영상시험을새binary의새실행결과로소급하지않는다.

수정본 실제 `python3 scripts/windows_current_native_ui_collect.py --proof docs/validation/evidence/windows-msvc-network-input-whole-20261008/result.json --output docs/validation/evidence/windows-msvc-network-input-native-20261008 --kind network` exit0. 원collector REVIEW_REQUIRED를보존하며root-review.json에서별도PASS_SCOPED. 첫/재실행 각ownedwindowmessage클릭,Home+문자삽입,End+Backspace,Tab→관측적용의UIAfocus,eth0/../invalid/빈값readback·적용·error표시/해제와Light/Dark·앱150/200를확인했다. 두process naturalexit0/원Job0/privateclosed/inputDefault불변. root가원valid/200PNG를직접봤다. 총46PNG는단계별중복캡처를포함하며46개의독립기능이나전수입력인수가아니다. Windows망측정성공·선택영속성·물리입력/IME는false유지.

나머지입력의근거는 docs/development/input-coverage-audit.md에분리했고진단singleline3개는S10-diagnostic-singleline-input-consistency.md의미구현후속이다. 기존953후보의공개영상/비용/레이아웃전체시험을현재957CPU에서다시실행했다고표현하지않는다.

최종 `scripts/windows_msvc_notice_graph_probe.py`의currentwhole/configuration→network-input-graph 명령exit0,graph3/boundwhole4·source610·258graph를root가확인했다. `scripts/windows_msvc_candidate_package.py`에currentwhole/currentgraph와원installerSHA를명시하여새ZIP생성exit0. 실제argv는collector원configuration/result/log와root 실행기록을따른다. ZIP17,244,068B·466members·SHA256 `35cd7866634b2af640abdf69f51c8c4a95032290461bd6bbc9ceb218c655173a`:root CRC/원3exeSHA/PE·fixedmode/timestamp·notice258원문SHA/selectedmissing0/currentproof바인딩을검증했다. wholemissing43/legal/releasefalse유지. 네트워크입력수직교정의scoped완료이며진단후속/전수입력/전체P0는미완료다.

최종 `python3 scripts/check_docs.py` exit0: PASS869documents·required/local file links,114hash-matched upstream notices보존. 외부URL/heading anchor검증은checker범위밖이다. root/독립패키지감사도CRC/3exeSHA/PE/currentproof/notice/fixedmetadata를확인했다.
