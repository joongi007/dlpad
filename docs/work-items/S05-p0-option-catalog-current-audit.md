# S05: 현재 P0 노출 옵션 catalog 읽기 감사

- 상태: completed (읽기 감사; R07 누락 수정·GUI 인수 별도)
- 분류: review
- 우선순위: P0
- 요구사항: R07
- 담당/수정 파일: soak_recovery_audit — 이 문서만
- 의존성: options catalog·CPU exposed UI/plan, 도구·옵션 계약

## 범위와 인수 조건

README/PRODUCT/CONTEXT→R07→INTEGRATIONS/ADR 순서와 실제 catalog/CPU UI를 대조한다. 모든 노출 주요옵션의 설명·예·기본·충돌·버전 또는 미확인을 검사하며 실제 source라인/SHA와 최소수정파일목록을 보고한다. Rust/Cargo/기존docs/GUI/manager 실행·수정은 없다.

## 현재 source 기준과 노출 범위

CPU entry는 accessible_cpu_main.rs의 동일 accessible Candidate/DownloadForm을 사용한다. 다운로드 옵션은 품질(Best/Height/Audio), 구간 시작/끝, 빠른/정확 cut, 자막3언어와자동허용, advanced3개, FFmpeg선택과session cookie선택이다. URL·tool·save root도 같은 form입력이다. 창버튼/appearance/metrics는R07다운로드옵션이아닌별도설정/액션으로분리했다. Config 가져오기/자유extractorargs/FFmpegcodec등은노출하지않으므로P1미구현을P0옵션누락으로세지않았다.

| 파일 | 감사 당시 SHA256 |
|---|---|
| crates/core/src/options.rs | 1d761e942cf4f0656fe9525e62becacf1079206c3a4b478404b79812b4490b7f |
| crates/app/src/accessible_options.rs | cbbdfb5a7dd5b3957a71a8df629e749184e57618fd5657e73e9cb73fa0452dd8 |
| crates/app/src/catalog.rs | 3484dc5e6ee4d41808748b60df5a0d125ab0f6293eda1332c07f0f242a6cca0f |
| crates/app/src/accessible.rs | 7b908393410ba72df61fb283368ba62590755f992ad161a0bf6a5be3d84b83cd |
| crates/app/src/accessible_plan.rs | eed80ed357f9c7d75f790705b6728296dc2090c087468de4564d6df67789d53a |

## 5요소 노출 검수

R07은모든노출옵션의설명/예시/기본/충돌/버전또는미확인을요구한다. 아래충족은현재source출력경로검수이며이번GUI가시성실행PASS가아니다.

| 옵션 | 실제 근거 | 판정/누락 |
|---|---|---|
| 속도 제한 | core options.rs214–223, accessible_options.rs169–173/198–213 | 5요소모두출력. 기본제한없음/예1048576/충돌없음/검증되지않음. |
| 재시도 | core224–233, 동일help | 5요소출력. 앱기본3과plan args일치,입력값은동일key한번만override. '앱재시도정책과별개'는job재개와구분하므로충돌없음과같지않다. |
| 조각 동시성 | core234–243, 동일help | 5요소출력. 예/기본1/부하비용/충돌없음/미확인. 빈값은upstreamdefault에의존하므로선택버전default확인전계속미확인. |
| 자막 언어 및 자동허용 | core178–187, accessible_options.rs127–158 | 설명/예/기본저장안함·자동off/구간충돌/설치버전미확인모두출력. 실제core342–357의구간거절과일치. |
| 품질 Best/Height/Audio | accessible_options.rs103–115, catalog204–209 | Summary와높이오류만출력. 명시defaultBest/height편집기1080,선택예시,충돌/지원범위,미확인버전표시누락. 선택상태/편집초기값만으로help5요소를대체하지않는다. |
| 구간 시작/끝 | accessible_options.rs116–117, catalog42–43 | Fieldlabel만있고시간예시/양쪽blank전체default/자막충돌/미확인버전설명누락. '시작비우면전체영상'은끝만입력한경우core가거절하므로양쪽비우기라고명확히해야한다. |
| 빠른/정확 cut | accessible_options.rs118–123, catalog100–102/44–45 | 키프레임/재인코딩비용있지만빠른default/구간있을때만적용/예/자막충돌/미확인버전표시누락. 정확선택+전체영상일때cutargv없다는core정책을설명해야한다. |
| FFmpeg 경로 | accessible.rs814, catalog113–114 | CPU는FFMPEG_HELP를호출하지않아설명조차누락. main.rs1155에만사용돼기존host검증으로CPU를대체하지못한다. PATH빈default/절대경로예/앱예약key충돌/확인버전또는미확인표시필요. |
| Session cookie 파일 | accessible.rs815–816, catalog230–231 | source보존/이번작업/재개새선택/로그생략summary는있지만명시default없음/합성경로예/OS지원·예약옵션충돌/미확인버전표시누락. R02인증privacy는별도이며R07도움말누락과구분. |
| URL·yt-dlp·저장 위치 | accessible.rs811–813 | 다운로드정책의입력fieldlabel만있다. R07을모든입력까지엄격적용하면HTTP(S)예/필수·저장위치기본·외부도구선택/예약key·미확인표시추가가필요. 최소주요옵션수정과함께명확한입력help를넣어해석차이를없애는것을권장. |

## 정책과 실행 계획 대조

core options.rs314–321은Best→bv*+ba/b,Audio→bestaudio/best,Height→상한format이다. catalogQUALITY_HELP의오디오fallback/변환없음설명은정책과일치한다. core363–377은구간에만download-sections/keyframecut을추가한다. core342–357자막구간거절,379–388중복key거절/retryoverride도UIbuild의options.plan검증과일치한다. 실행계획은accessible_plan.rs44이후coreplan을사용하고URL전체생략/256KiB/Pagination이설명과일치한다. 쿠키는coreplan에없고worker에서별도lease argv로전달되므로plan에쿠키경로가안보이는것은privacy범위이며이를완전실제argv라고단정하지않게후속help를검토한다.

INTEGRATIONS의since/until/tool/value_type/sensitive/upstream_help schema는현재OptionHelp203–211에모두존재하지않는다. R07 P0의versionorunverified는현재고급/자막에충족하지만확장schema/P1help snapshot은미구현이다. 임의upstream버전/설명을만들거나catalog의미확인을검증완료버전으로바꾸지않는다.

## 최소 후속 수정 파일과 인수

각파일은순서대로독점배정하고같은파일동시수정하지않는다.

1. `crates/core/src/options.rs`: typed QUALITY/RANGE/CUT 도움말 OptionHelp상수를추가. 기존advanced CATALOG3개배열을확장해입력매핑을깨지않는다. OptionHelp schema변경이나argv정책변경은불필요하다.
2. `crates/app/src/accessible_options.rs`: 해당typedhelp를선택control옆에출력. 기존공통help가5요소를출력하므로재사용한다.
3. `crates/app/src/catalog.rs`: 경로/쿠키/URL·저장입력의bounded고정help·default/example/conflicts/unverified문자열을중앙관리한다. 기존range기본label을양쪽blank정책과맞춘다.
4. `crates/app/src/accessible.rs`: FFmpeg등field옆중앙help출력. 이전FFMPEG_HELP누락을해결한다. Root현재공유owner와조정해할당한다.

인수는모든실제노출옵션의5요소source검사와기본/예시가실제plan정책과맞는meaningful회귀,CPUprivateGUI에서도움말가시성/scroll/키보드접근을확인한다. 새height나field설명으로normal960페이지길이가늘어나므로가시성이별도gate다. 고급3/자막의기존실행옵션/SQLite/argv를유지하고실제설치버전검증없는옵션은명시미확인으로둔다. OS별GUI범위는실제검증한것만보고한다.

## 실제 실행과 남은 결정

README/PRODUCT/CONTEXT/R07/INTEGRATIONS/ADR0003및catalog/CPUsource와기존S05workitem을읽고 rg/nl/sha256sum으로라인·SHA를기록했다. Rust/Cargo/GUI/네트워크도구조회는실행하지않았다. 이번완료는읽기감사이며R07전체완료가아니다. 핵심누락은CPUquality/range/cut의5요소및FFmpeg설명,후속cookie/path도움말이다. Root가최소파일담당을조정해구현/실제인수를수행한다.
