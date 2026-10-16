# S05: 기본 UI 옵션 도움말 누락 읽기 감사

- 상태: completed (읽기 감사; 기본 UI 구현 별도)
- 분류: review
- 우선순위: P0
- 요구사항: R07
- 담당/수정 파일: soak_recovery_audit — 이 문서만
- 의존성: 새 app catalog metadata, root CPU compile/source freeze

## 범위와 인수 조건

기본 main.rs 품질/구간/cut/FFmpeg의 새 중앙도움말 실제출력을 확인하고 main.rs 단일파일 후속수정위치/scroll·키보드영향을 기록한다. Rust/Cargo/source수정·GUI실행은 없다.

## Source 근거

main.rs SHA256 `41cf573fc420cdaf76685df94602f2e7ff687e2a9de82e96926b3bd444abee1d`,catalog.rs `2760ddf24784282d4d7db84cfe4528f4ae911dd7813e9b73d8ebcd6f4d681f36`를읽었다. CPUhelp보완이기본UI로자동연결되지는않는다.

| 항목 | 현재 main.rs 출력 | 누락/재사용 위치 |
|---|---|---|
| FFmpeg | 1149–1155 field/FFMPEG_HELP summary | 1155 뒤 FFMPEG_DEFAULT_EXAMPLE/CONFLICT_VERSION 추가로5요소완성가능. |
| 품질 | 1156–1159 QUALITY/숫자field | 기본UI는0:Best/144~4320Height만지원,Audio선택없음. CPU QUALITY_DEFAULT_EXAMPLE/CONFLICT_VERSION은Audio를설명하므로그대로재사용하면미지원동작을안내하는버그다. 기본UI용중앙numeric품질metadata가추가로필요하다. |
| 구간 | 1160–1167 label+placeholder예시 | RANGE_HELP/DEFAULT_EXAMPLE/CONFLICT_VERSION을끝field뒤출력. 새양쪽blanklabel은이미중앙catalog로반영됐지만5요소help는없다. |
| 자르기 | 1168–1170 pick_list/CUT_COST | CUT_DEFAULT_EXAMPLE/CONFLICT_VERSION을cost뒤출력. 현재Fast/Accurate정책과일치. |
| 자막/자동 | subtitle_controls995–1041 | SUBTITLE_HELP의summary/default/example/tradeoff/conflict/version모두기존출력. 유지. |
| 고급3 | advanced_controls1045–1105 | CATALOG metadata기존출력,변경불필요. |

기본UI는cookie source를노출하지않아CPUcookiehelp를기본main에추가할이유가없다. URL/tool/root별도입력help는이좁은후속품질·구간·cut·FFmpegscope밖이다. main의quality parser924이후0/height와기본quality초기값을통해CPUQualityMode/Audio와다름을확인했다.

## 최소 후속 담당과 UX 영향

기본 `Page::Downloads`는1124이후단일scrollable column이고입력순서가FFmpeg→quality→range→cut→subtitle→advanced→Start다. 가장작은main.rs수정은각field/기존summary옆text(size13)로중앙metadata를표시하는것이다. 비interactive text는tab입력순서를늘리지않지만column길이가늘어Start/Plan까지scroll이필요하므로기존privateGUI키보드/scroll인수가필요하다.

접힘을추가하려면기본iced는CPUegui collapsing을사용할수없으므로별도boolean/Message/button분기가필요하다. 기존advanced_open/ToggleAdvanced패턴을활용할수있지만작업범위를늘린다. 처음에는bounded고정설명행을기존scrollable에넣고가시성을검증하는최소수정을권장한다.

Root가main.rs단일담당을배정하되numeric품질의중앙문구는catalogowner와조정해야한다. QUALITY_DEFAULT_EXAMPLE을그대로사용하지말고새DEFAULT_QUALITY_HELP/DEFAULT_EXAMPLE/CONFLICT_VERSION을catalog에독립상수로추가하는것이최소안이다. 공통RANGE/CUT/FFmpeg metadata는직접재사용가능하다. CPU문구나coreargv/schema를이기본후속에서바꾸지않는다.

## 실제 검증과 잔여

rg/sed/sha256sum읽기만수행했다. 신규workitem외파일변경·Rust/Cargo/GUI실행없음. Source검수로실제help5요소누락을확인했으며기본UI구현/메타데이터정합성/원래scroll·keyboard/privateGUI는후속인수다. RootCPUcompile진행과기본UI인수를서로대체하지않는다.
