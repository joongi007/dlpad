# S00: 자막 선택과 체크박스 일관성

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: 주 담당 crates/app/src/accessible_options.rs, 본 문서. 다른 에이전트는 이 파일을 수정하지 않는다.
- 선행 작업: S00-ui-current-priority-review-20261012, 기존 ui_components choice/checkbox

## 사용자 결과

같은 자막 선택 그룹의 네 값이 동일한 선택·호버·포커스 표현을 사용하고 자동 자막 체크박스는 다른 제품 체크박스와 같은 모양을 사용한다.

## 계약과 범위

DESIGN.md의32px 선택·상태 구분 규칙을 적용한다. 기존 choose_subtitles와 enabled/changed 정책, 중앙 문자열을 유지한다. schema·Cargo·의존성 변경은 없다. 공통 컴포넌트를 재사용하므로 별도 런타임 서비스·메모리 캐시를 추가하지 않는다. 기존 기능과 native 접근성 의미를 유지하는지 실제 검증해야 한다.

## 인수 조건

네 값의 단일 선택·자동 자막 값 유지·None에서 checkbox 비활성·구간 충돌 정책을 검증한다. Light/Dark 고배율에서 실제 선택·호버·포커스 및 wrapping을 확인한다. 새 scope로 생기는 ID가 프레임 간 안정적인지 확인한다. 코드 통합만으로 시각 인수 완료를 주장하지 않는다.

## 검증과 handoff

독립 감사 /tmp/dlpad-ui-current-priority-audit/REPORT.md를 주 담당이 실제 callsite와 대조했다. 새 product 수정의 Windows 빌드·기능·화면 검증은 아직 미실행이다. 기존1145 whole과 설치 ZIP은 이번 수정 전 snapshot이다.

Windows whole session87308에서 fmt·clippy·test는 각각 actual exit0, source_unchanged/private guards를 확인했다. test.log의43harness 합계는1145/0/30이다. release는 아직 terminal receipt가 없고 동일 handle을 계속 관측한다. 현재 추가 회귀 후보는 /tmp/dlpad-subtitle-control-regression/regression.rs에 준비했으며 원본 검사 종료 전 제품 source에 삽입하지 않는다. 이 후보는 실제 Options.show/AccessKit Click의 단일 선택·auto flag·비활성 무반응을 검사하며 OS 물리 입력 시험이 아니다.

Windows whole87308은 release까지 actual exit0로 종료했고 별도 검토35729도 exit0로 raw4·43harness1145/0/30·현재source·artifact·GNU/lock 보존을 확인했다. CPU9,690,624B SHA a37fcd0ade50e984a47326b7550654523d24a6f8b562e11f8603564f413aea06이며 의존성 변경/크기 증가는 없다. 그 뒤 actual Context 회귀를 통합했고 `python3 /tmp/dlpad-subtitle-options-targeted.py` session20699는 exit0이다. 실제 app 두harness에서 새 회귀가 각각1회 성공, Light/Dark의 선택·auto·disabled·AccessKit·ID 안정성을 확인했다. 원본 raw/private naturalJob0/source 보존을 root-review.json으로 좁게 인수했다. 전체 OS 물리 조작·고배율 PNG·사용자 디자인은 미검증이다. 추가된 테스트까지 포함하는 fmt/clippy 및 source proof 정합성과 Light/Dark200 private observer 실행은 후속이다.

고배율 private GUI 후보는 /tmp/dlpad-subtitle-native-probe-plan/에 준비했다. 주 담당의 읽기 검토에서 seed와 선택 theme를 같게 넘기면서 SQLite 생성을 기대하는 fixture 오류를 발견하여 실행 전에 수정하도록 했다. Light/Dark 각각 반대 seed에서 실제 선택 후 같은 data로 재시작하며, checkbox ACK와 language 선택 강조의 직접 PNG 판정을 구분한다. 새 회귀 source의 whole14226은 fmt/clippy 통과, test/release 종료 대기 중이다.

새 회귀 포함 whole14226의 test는 actual exit0,43harness1147/0/30이며 release는 원 handle 관측 중이다. native observer의 반대 seed→Light/Dark와 전체 geometry 미검증 flag를 주 담당이 확인했고 Python 구문 검사를 통과했다. 독립 검토 담당 diagnostic_inputs는 /tmp/dlpad-subtitle-native-independent만 작성하며 원 raw/소스/CPU/phase·SQLite·cleanup를 대조한다. 제품 파일의 동시 수정은 없다.

whole14226·독립59993은 actual exit0로 종료했다. source/원 raw4/43harness1147/0/30/GNU·lock 보존을 대조했고 CPU9,690,624B SHA a81354e3de59f068c775b9f5f83ccc2e5935336436a2ee768e403626523e3676이다. release CPU는 새 회귀 포함 source의 pin으로 사용한다. private Light200 session71088, Dark200 session9956을 각 fresh output·독립 소유 data/desktop으로 실행했다. 결과·전체 glyph·선택 시각은 아직 인수하지 않았다.

Light71088/Dark9956은 둘 다 native exit1로 종료했다. 첫 자동 자막 Checkbox의 도달 시도32회에서 target_client_y1218/anchor_client_y86이 계속 같았으며 원 실패 PNG에는 품질 선택과 열린 구간·자막 제목만 보였다. 성공·전체 UI 인수로 표시하지 않는다. 원본 로그·캡처·JSON을 그대로 보존했고 subtitle-native-reachability-failure-20261012.json에 root의 실제 관찰을 기록했다. private desktop 종료/input desktop 보존은 확인했으나 app 자연 종료 전체 인수와 구분한다. 다음은 실제 본문 wheel 소유권과 fresh provider 좌표를 기록한 v2 관측기로 재현해 제품/관측기 원인을 구분한다. 제품 코드는 이 실패만으로 변경하지 않았다.

원 Light 실패 PNG도 직접 확인했다. diagnose의 재현→가설→관측→수정 순서를 적용하며 docs/agents/의 기존 local tracker/single CONTEXT 설정을 유지한다. anchor/body 불일치, private pointer 이탈, 제품 parent reveal의 세 가설과 예측을 실패 기록에 추가했다. 기존 성공 Library200 helper의 원 body 중앙·번갈아 mousemove·bounded adaptive wheel을 v2 관측기에 참고한다. 픽셀 정상 여부로 실패했던 조작을 성공으로 바꾸지 않는다.

Dark200 v2 session4129을 fresh output으로 실행했다. 원 v1실패를 보존하고 exact current whole/CPU pin을 재사용한다. 후보는 본문 중앙+번갈아 mousemove와 fresh exact-name/type/PID query,32회 상한, actual displacement 적응 delta1..120을 기록한다. 전체 체크박스 가시 영역과 언어 선택 전체 glyph 인수는 분리한다. 원 codepage 암묵값 대신65001을 명시했다. v2의 body clip은 nav/footer의 실제bounds에 제품 padding을 적용한 보수적인 추정이며 clip 자체를 OS API로 독립 관측했다고 주장하지 않는다.

Dark200 v2 session4129은exit1(후속 관측기 오류)로 종료했다. 원 s10-scroll의 Checkbox top1192→1164→…→346, height64, body132..412로 실제32회후 full_control_visible=true를 확인했다. 이어 같은 output에 S10Click이 create-new s10-action-before.png를 다시 쓰려다 IOException이 발생했다. 따라서 reachability는 좁게 개선됐으나 language 전체 흐름은 아직 성공하지 않았다. 원 v2를 보존하고 /tmp/dlpad-subtitle-native-probe-plan-v3에서 각 SubtitleClick에 새 소유 action 폴더를 사용하도록 수정했다. 제품 source 변경은 없다.

Dark200 v3 session94819은 원32회 도달 상한에서 exit1이다. 체크박스 top1192→353/height64로 실제 이동했으나 보수적인 body bottom412를5px 넘었다(417). v2는 같은 앱/규칙에서346으로 들어왔으므로 smooth-scroll/time 샘플의 차이를 성공으로 소급하지 않는다. v3 실패를 보존하고 v4 관측기는 전체bounds 규칙을 유지한 채 상한만64회로 늘려 동일 closed-loop 시험을 계속한다. source·최대wheel120·소유PID·private/전체500초 제한은 유지한다. clipping을 허용하도록 기준을 낮추지 않았다.

Light200 v4 session25785도 별도 fresh output으로 실행했다. source/DB/desktop은 Dark6341과 분리되며 제품 source를 고정했다. 독립 v4 검토기 /tmp/dlpad-subtitle-native-independent-v4는 원본16 subtitle phase PNG·2 lifecycle·10 action 캡처와 natural app/Job/private 종료 및 반대 seed의 실제 theme DB를 대조한다. Checkbox fullbody는 엄격히 유지하며 선택 언어의 pixel 판정은 주 담당이 별도로 수행한다. 순수 검토기8검사는 후보 검사이며 native UI 완료를 뜻하지 않는다.

Dark6341와 Light25785 v4는 actual terminal exit0/REVIEW_REQUIRED로 끝났다. 각16개 자막 phase 및 first/restart 흐름을 수집했고 독립 검토가 raw toggle·원 source/CPU·자연 app/Job0·private cleanup·read-only theme DB를 대조한다. root는 Dark6/Light8개 PNG를 직접 보고 네 언어의 선택 강조, checkbox on/off·disabled·full label, footer 겹침 없음에 scoped pixel PASS를 기록했다. 모든32개 PNG 직접 인수/물리 입력/전체 UI/UX는 아니다.800폭과 hover/pressed 실제 native 인수는 남아 있다. 설치 ZIP은 여전히 이전 후보이므로 새 UI를 설치본에 포함했다고 표현하지 않는다.

800×640/앱200% v5의 observer diff를 주 담당이 읽어 originalHWND·실제 decoration inset·SWP_NOACTIVATE와 각 phase의 exactclient 요구를 확인했다. fresh Dark74999/Light98282를 별도 owned root/desktop에서 실행했다. 각 phase가 네언어 fullbody geometry와 selected geometry를 기록하되 전체group 성공을 추정하지 않는다. 독립 담당은 /tmp/dlpad-subtitle-native-independent-v5 후보만 준비하며 제품 source 변경이 없다. source 고정 중 품질Focus 후보를 제품에 삽입하지 않는다.

800200 Dark74999/Light98282 모두actualexit0로 종료했고 독립84229/55316도exit0이다. 원 source/CPU/viewport800640·각phase64px checkbox/body·언어geometry flag·raw/private cleanup/read-only DB 대조를 root-independent-native-review.json에 보존했다. root는 양테마 한영+autoOn PNG를 직접 보았다. 이후 footer 간격 회귀를 추가하므로 기존1147 whole과 자막 source proof를 후속 테스트 포함 source로 소급하지 않는다. 새 source의 추가검사는 별도 진행한다.
