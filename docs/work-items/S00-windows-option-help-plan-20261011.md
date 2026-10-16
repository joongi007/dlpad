# S00: Windows 고급 숫자 입력과 실행 계획 인수

- 상태: completed
- 분류: verification
- 우선순위: P0
- 요구사항: R07, R08, R11, R13
- 담당/수정 범위: root 실행/통합/검증; diagnostic_inputs는/tmp전용새observer/collector후보. 대상 scripts/windows_option_help_plan_probe.cs, scripts/windows_option_help_plan_collect.py와회귀/문서; 제품수정필요시root가별도조정.
- 선행 작업: 기존공개walkthrough의시간/품질/고급펼침과진단singleline3곳표시인수. 새Windowsfault전체검증.

## 사용자 결과

고급숫자3곳의입력·오류·유효값·도움말·실행계획의일치성을실제Windows화면으로검증한다. 펼침만확인한기존증거를값입력인수로표현하지않는다.

## 계약과 범위

별도privateDesktop/이번작업의tmp합성tool/data만사용한다. URLfixture.invalid이며실제계정/영상/외부망은사용하지않는다. 다운로드시작없이plan텍스트만검증하며clipboard/globalfocus/visiblefallback은없다. 새로운collector파일만담당하고공유Cargo/schema/lock은root조정한다. customtoolpath·물리IME·실제옵션다운로드영향은이작업의완료범위가아니다.

## 인수 조건

속도제한0→1048576, 재시도21→0, 조각동시성0→1의actualinput32px/readback/오류→해제·help기본/예/충돌/설치버전표시와plan의각옵션exactonce를확인한다. 자연exit0/ownedJob0/privateclosed/inputdesktop불변·합성원본/보호자료보존을root대조한다. 단위PASS로시각검증을대체하지않는다.

## 검증과 handoff

현재읽기감사에서는고급펼침만실제증거가있었고숫자3곳/help/plan조작의원증거가없었다. /tmp후보준비중이며실제Windows미실행이다. 초기Gitbundle2044cbab와설치ZIP은이후테스트추가전검증snapshot으로보존한다.

추가 재현: `windows-msvc-option-help-plan-first-20261011`은 numeric6/help/argv exactonce와자연종료를통과했지만plan.png는viewport아래여서시각PASS아님. `...-focus-...`는readonly Focus거부, `...-scroll-...`는readonly전체396px bounds를완전히보이게하는observer조건과휠경로로실패했다. 두실패는자연exit/Job0/privateclosed 보존. 제품의광고된ScrollIntoView를처리하지않는별도gap을수정하며readonly포커스를추가하지않는다. 범위에 accessible.rs/show_plan과actualglyph회귀를추가한다.

회귀초기실행 `windows-msvc-plan-scroll-red-20261011`은root의test삽입이bounded_font_tests에들어가Fixture/wait_finished범위compiler실패였다. 제품실패RED로계산하지않는다. tests모듈위치교정후fresh `...-red-resolved-...`로재현한다. 최초runner의TEMPslash표기는동일Windows경로의정규화대조로교정했으며외부TEMP를허용하지않는다.

selector시도들은absolute합성tool/path 없는invalidplan 및readonlyrole가TextInput이아닌MultilineTextInput이라는fixture조건교정이었다. 의미있는제품RED는 `windows-msvc-plan-scroll-glyph-red-20261011`의actualglyph≥120px visibleviewport assertion실패다. root는pagepersistentID/exactScrollIntoView request를innerpreview/parent각각에전달하는show_plan좁은수정을통합했고GREEN검증중이다. readonly/noFocus/상한은유지한다.

`windows-msvc-plan-scroll-glyph-green-20261011` actualCPUbin1PASS: rawterminal/pins/glyph/readonly/noFocus를 root가 독립 대조해 root-review PASS_SCOPED。full `windows-msvc-plan-scroll-whole-20261011` 진행중. 광고된ScrollItemPattern을호출하고전체396pxtext가아닌boundedpreview clip을검사하는새observer/collector2파일을root가scripts에통합했다. 아직새바이너리GUI미실행이다.

현재전체 `windows-msvc-plan-scroll-whole-20261011`: fmt/clippy/test/release 실제PASS,42harness1013/0/26,618source. CPU9537024B SHA b8434e702ab0282af5f406471bc4eb360d3f2cef429a3fa54eac4fd1e90c7d6f; CLI/installer은이전whole과byte동일. root가4rawterminal/sourcefreeze/artifact를독립대조했다. 새실제ScrollItem GUI인수는진행중이다.

### 최종 인수

`python3 scripts/windows_option_help_plan_collect.py --proof docs/validation/evidence/windows-msvc-plan-scroll-whole-20261011/result.json --output /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-option-plan-native-scroll-20261011` 실제 실행 PASS_SCOPED. root는 numeric6/help/argv3flag exactonce, readonly ScrollItemPattern, 실제 plan PNG의 글자 표시, raw natural0/Job0/privateclosed/inputdesktop 불변, original pins와 sentinel을 독립 검토했다. [원본 결과](../validation/evidence/windows-msvc-option-plan-native-scroll-20261011/result.json), [root 인수](../validation/evidence/windows-msvc-option-plan-native-scroll-20261011/root-review.json).

제품 변경: 읽기 전용 실행 계획의 광고된 ScrollIntoView를 실제 처리한다. 수정 파일은 crates/app/src/accessible.rs, scripts/windows_option_help_plan_probe.cs, scripts/windows_option_help_plan_collect.py다. 새로운 의존성·Cargo.lock·schema 변경 없음. 물리 포인터·IME·실제 다운로드 옵션 효과·custom tool path 입력은 이 작업의 인수 밖이며 완료로 주장하지 않는다. 화면의 넓은 빈 공간과 계획 뷰 폭은 후속 UI 밀도 점검 대상으로 남긴다.
