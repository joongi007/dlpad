# S00: 스크롤 후 시작 불가 원인 접근

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: root — crates/app/src/accessible.rs, crates/app/src/catalog.rs, crates/app/src/media_workspace.rs, 이 문서
- 선행 작업: S00-diagnostic-single-viewport-frame-20261012의 source 고정 검증 종료

## 사용자 결과

상세 설정까지 스크롤한 사용자가 비활성 다운로드 시작 버튼의 원인과 해결 방법에 접근한다.

## 계약과 범위

DESIGN.md의 disabled 이유와 해결 동작 계약을 따른다. 기존 원인을 재사용하며 gate·네트워크·도구 정책은 변경하지 않는다. 고정 action에 원인을 연결하고 시작 조건 확인으로 URL·이유 위치를 함께 공개한다. footer에 긴 중복 설명을 늘리지 않는다.

## 인수 조건

URL 없음·도구 없음·작업 중 상태의 원인을 keyboard와 접근성으로 읽고 해결 위치로 이동할 수 있다. enabled 전환 시 이전 원인이 남지 않는다. Light/Dark 좁은 창에서 footer·시작·재개 동작이 겹치지 않는다.

## 검증과 handoff

현재 제품에는 원인 description·시작 조건 확인·URL/이유 공개·후속 휠의 재공개 취소가 구현돼 있다. source-cluster snapshot에서 Windows 전체1155/0/30과 Light/Dark800×640/앱200% first/restart 실제 GUI·독립 provider/DB 검토 및 직접4PNG 검토를 통과했다. local/enabled 전환의 별도 actual case, 도구 오류의 native 화면, 물리 입력과 전체 UI/UX 인수는 남아 in-progress를 유지한다. 아래 실패/수정 기록은 구현 과정의 증거이며 현재 상태를 대신하지 않는다. 후속 source-first UI 변경은 별도 작업으로 재검증한다.

- root 실제 RED session92055 exit101: 컴파일 후 actual Candidate에서 조건 확인 버튼0개/기대1개로 실패했다. 단순 compile 오류가 아니다. 원 source 불변/강제 종료 없음. 기존 reason/gate를 연결하는 최소 후보를 통합하며 parent busy 상태에서는 안내 버튼도 disabled인 제한을 다음 인수에서 별도 보완한다.

- GREEN56734/instrument57964/immediate24637/owner39756 모두 컴파일 후 실제 이유 glyph 관측에서 실패했다. 클릭 frame11과 gate URL_REQUIRED, media_error=None을 확인해 dedup 원인을 반증했다. 즉시 공개 및 중첩 영역 뒤 공개만으로 해결되지 않았으므로 원인 확정으로 쓰지 않는다. session36680에서 실제 target와 최종 offset을 추가 계측 중이다. 모든 실패를 보존하며 이 기능은 미완료다.

- 계측36680은 새 Context 재진입 잠금으로10초 debug deadlock 실패했다. 제품 원인 증거로 계산하지 않는다. ctx frame 값을 data_mut 진입 전에 읽도록 계측을 수정하고 새 trace-v2로 재실행한다.

- trace-v2 session12495 exit101: 실제 clickframe11 target Rect93..125는 당시 보였으나 최종 offset240으로 이동했고 reason glyph는 사라졌다. 이전 휠 이동 중 클릭한 단일 frame request로 공개를 마칠 수 없음을 확인했다. 중첩 처리 개선만으로 해결되었다고 주장하지 않는다.
- pending 공개를 대상 전체가 보이고 잔여 smooth wheel이 끝날 때까지 유지하며 새 raw wheel은 취소하도록 수정했다. 실제 settle session19177 exit0: 두 app harness에서 이유 glyph fullclip·원 gate·worker 미생성 검사 통과.
- 새로운 휠 이후 실제 outer ScrollArea offset 증가와 pending 종료를 추가로 검사하는 session52925 진행 중. busy 원인 접근·전체 검사·새 native 시각 인수는 남아 있다.

- wheel session52925 exit0: 두 app harness의 실제 outer offset이 이유 공개 후 새 휠에서40logical 이상 증가했고 pending 요청은 종료·미재활성화했다. 원 gate/worker 보존도 통과. root-scoped-review.json에 좁은 검토를 기록했다. OS native·busy 전체 인수는 아니다.

- busy RED86746 exit101: 원 Worker 타입에 소유 합성 대기 thread를 연결하고 시작 ACK를 받은 상태에서 설명 버튼 disabled를 실제 Candidate로 재현했다. 실패·성공 모두 원 thread 종료/join을 수행했다. 다운로드 I/O 검증이 아니다.
- 설명만 활성화하고 원 Start/Resume/Recover에 이전 actions_enabled gate를 각각 유지했다. busy GREEN91655 exit0: 두 app harness에서 전체 이유 공개/새 휠·pending 종료/원 gate 유지/살아있는 소유 worker에서 설명 enabled 검사 통과.
- 새 전체 Windows collector 시작, 모든 처리 중 상태·원 preview error dedup 및 실제 OS native 시각 인수는 남아 있다. 전체 UI/UX 또는 P0 완료로 확대하지 않는다.

- 전체 Windows42904 exit0, 독립 원본 검토93073 exit0:43 harness/1155성공/0실패/30ignored, 현재 CPU9,704,960바이트 SHA `c87479becad0922cb6148d0e965e846bb31601f8a28e53902a1496bbb0df4238`. 이전9,693,184대비11,776바이트 증가, 의존성/Cargo 변경 없음. 실행 비용 전체 인수는 별도다. root-review.json 및 원 로그를 보존했다. 실제 native 조건 버튼 검증과 예외 상태 검토는 남아 있다.

- 실제 Light81913/Dark73927 GUI 시험 모두 native FullDescription 속성 조회에서 실패했다. .NET AutomationProperty.LookupById 미지원으로 앱은 원 Job 강제 정리되어 자연 종료 인수는 아니다. 실제 조건 공개/후속 휠 단계 미실행, 두 실패 기록 보존. 직접 native UIA 속성 조회 후보를 별도 준비하며 HelpText/caption으로 성공을 대체하지 않는다.
- 독립 source 감사에서 페이지·로컬·enabled 전환의 pending 미종료, dedup 이유 대신 버튼만 target, oversized row의 contains_rect 완료 조건을 확인했다. 실제 분기 회귀 없이 전체 상태 성공으로 확대하지 않는다. 화면 전환 회귀63377은 테스트 catalog 상수 오기 compile 실패이며 수정한80521에서 실제 재현을 진행한다.

- retirement RED80521 exit101: 실제 AccessKit 조건 Click→Settings Click으로 page 전환을 확인한 뒤 pending 요청이 남아 실패했다. 원 worker/metrics 종료 보존. source 수정은 download page를 떠남/local source/current reasonNone에서 pending을 종료한다. GREEN61865 진행 중. 도구 오류 dedup target은 실제 media 오류 Label rect 관측으로 별도 보완할 예정이다.

- retirement GREEN61865 exit0: 두 app harness에서 actual Settings 전환→pending 종료→Download 복귀 미부활, 이전 fullreason·새 휠·합성worker gate 회귀 통과. local/enabled 전환은 수정 경로만 있으며 아직 별도 actual case 인수는 아니다. 현재 source의 새 전체 검사는 dedup/큰 target 보완 뒤 묶어 실행한다.

- dedup RED86878은 합성 저장 폴더가 비어 location gate가 먼저 막아 테스트 setup 실패였다. 소유 fixture 폴더를 명시한 RED66107은 실제 public preview preflight의 yt-dlp 없음 오류와 원 gate 일치 후 exact 오류 glyph가 그려지지 않아 실패했다. 도구 실행·실제 네트워크 없음.
- media_workspace의 실제 Label rect를 매 show 초기화·caption 비교해 노출하고 부모 공개 target에 우선 연결했다. dedup GREEN91343 exit0: 두 app harness에서 exact 오류 전체 glyph 클립·페이지 retirement·기존 reason/새 휠·원 worker gate 회귀 통과. 전체 source 검사와 새 native UIA 조회 시험은 다음이다.

- 새 전체21216 exit0, 독립6431 exit0:1155/0/30·43 harness 및 source/raw/artifact 원핀 대조 통과. 현재 CPU9,707,520바이트 SHA `68c346d04a4c6846f63c5c6270a386573d8cddbd9180501b45811cc8ef59f0d6`, 이전 대비2,560바이트 증가. 의존성/Cargo/schema 변경 없음.
- 새 native COM 속성 조회 v2 Light15909/Dark82379 실행 시작. 기존 FullDescription unsupported 두 실패는 보존하고 HelpText/기대 문구 대체 없이 actual native 속성을 조회한다. 아직 실제 GUI 성공 주장은 없다.

- native v2 Light15909/Dark82379 실패: 실제 COM FullDescription30159 조회는 원 HWND/PID/opaque RuntimeId 대조와 정확한 사유 값으로 성공했으나 이유 공개 뒤 URL control top-16/full_control_visible=false였다. 이유는 full_control_visible=true였다. root가 실제 Light start-conditions-revealed.png 직접 확인. 두 앱은 실패 정리로 강제 종료됐으므로 자연 종료 성공으로 계산하지 않는다.
- UX를 수정해 이유 행을 URL 바로 아래에 배치하고 source label/input/reason union을 함께 공개한다. media tool error는 실제 오류 Label 우선 target을 유지한다. 실제 source-cluster85039 exit0: 두 harness에서 실제 source label/input rect가 body clip 안에 있고 이유 전체 glyph·dedup 오류·새 휠·페이지 retirement·소유 합성worker 실행 gate 검사 통과. 새 전체 검사 시작, native 재검증은 남아 있다.

- source-cluster 전체49443 exit0 및 독립1431 exit0:43 harness/1155성공/0실패/30ignored. CPU9,708,032바이트 SHA `186a545d8ec3c251558f3b0ee64248bee2b2db12d559e2f50c40d44c1aed7111`, 이전 대비512바이트 증가, Cargo/의존성 변경 없음. 새 Light/Dark private-v3 실행 시작, 원 v1/v2 실패 보존.

- v3 Light53344/Dark79629 실제 GUI collector exit0 REVIEW_REQUIRED. root가 Light/Dark first/restart 공개 PNG4장을 직접 확인해 URL label/input·이유 문구·footer 분리를 좁게 인수하고 root-scoped-pixel-review.json 보존. 전체 phase pixels/물리 입력 인수는 아니다.
- 독립22194/39757는 다운로드를 시작하지 않은 ownedroot의 absent jobs.sqlite를 요구해 실패했다. 제품은 startup에서 새 jobs DB를 생성하지 않는 기존 계약이며 확인한 data는 theme.json/theme.sqlite/instance.lock이다. 기존 실패 기록 보존; 새 검토기는 absent DB+sidecars와 existing readonly DB를 구분하고 absence를 count0로 조작하지 않는다. 검토 통과는 아직 보류다.

- 수정한 독립 검토 Light12342/Dark99148 모두 exit0 `PASS_RAW_PROVIDER_DB_ONLY_PIXEL_PENDING`. 원 native COM 설명/PID/HWND/RuntimeId·공개 URL/이유 provider geometry·후속 휠·테마 readonly DB·원 source/CPU/두 자연 종료/Job0를 검토했다. absent jobs DB와 sidecars는 전후 부재로 기록하고 counts=null 유지. 새 DB 생성 없음. 각 evidence에 root-independent-native-review.json 보존, 별도 root4PNG 좁은 픽셀 인수와 결합. 모든 phase·물리 입력·도구 오류 native·전체 UI/UX 완료는 아니다.
