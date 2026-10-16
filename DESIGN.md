# dlpad 디자인 가이드

## 기준과 적용 범위

제품 UI. 사용자는 낮에는 다른 작업 옆에 다운로드 목록을 두고, 밤에는 긴 영상 진행 상황만 짧게 확인한다. 기본 테마는 System이며 Light/Dark 모두 동등하게 지원한다.

2026-09-30에 다음 gtmux 소스를 직접 확인했다: `codebase/frontend/src/styles/tokens.css`, `global.css`, `lib/stores/theme.svelte.ts`, `lib/ui/Button.svelte`, `Input.svelte`, `Tooltip.svelte`. 아래 원본 값은 그 스냅샷이다. 웹 코드나 CSS 엔진을 가져오지 않고 의미를 Rust theme 타입으로 옮긴다. 2026-10-07 실제 소스 컴포넌트와 private GUI 캡처를 추가 대조했다. 1·2차 후보는 사용자가 재거절했다. 정적 캡처 검토를 상호작용 및 사용자 인수 완료로 취급하지 않는다.

## 색상

절제된 회색 표면과 파란 accent를 유지한다. 원본 충실도를 위해 순백·검정도 유지한다. 새 색을 임의로 추가하기보다 역할 토큰을 사용한다.

| 역할 | Light 원본 | Dark 원본 |
|---|---|---|
| background | #ffffff | #1e1e1e |
| surface | #ffffff | #2c2c2c |
| surface-secondary | #f5f5f5 | #383838 |
| foreground | #000000 | #f5f5f5 |
| muted | #6b6b6b | #9a9a9a |
| subtle | #9a9a9a | #6b6b6b |
| border | black 10% | white 10% |
| border-strong | black 18% | white 18% |
| accent/info | #0d99ff | #0d99ff |
| success | #22c55e | #22c55e |
| warning | #facc15 | #facc15 |
| danger | #e5484d | #e5484d |

원본 accent 위 흰 작은 글자는 대비 검사가 필요하다. dlpad 제안: primary 버튼 배경과 Light 링크에는 #006fbe를 별도 `accent-solid` 토큰으로 사용하고 #0d99ff는 선택·진행 표시로 보존한다. success/warning은 아이콘·마커에 사용하고 본문은 foreground를 사용한다. subtle은 필수 상태·본문 텍스트에 사용하지 않는다. 실제 배경 조합마다 일반 텍스트 4.5:1, 큰 글자·UI 경계 3:1을 검증한다.

## 타이포그래피와 치수

원본 system sans/SF Pro/Segoe UI 계열과 숫자용 mono를 따른다. 목표는 OS 폰트와 한글 fallback을 검증하는 것이다. 현재 Windows CPU 후보는 Segoe UI → Malgun Gothic → egui 기본 내장 폰트 4종 순서로 proportional 글꼴을 사용한다. monospace 계열은 유지한다. 시스템 글꼴 두 파일은 창 생성 전에 읽고 각각32MiB(합계64MiB)로 제한한다. 이 환경의 실제 Segoe959,752B·Malgun13,459,196B이며 Segoe 추가 로딩 비용은 약0.92MiB다. 2026-10-12 후속 소스는 Windows에서 Segoe UI Symbol을 단색 symbol/emoji fallback으로 추가한다. 기존 UI·한글 글꼴 뒤에 배치하고 테스트 폰트 override에서는 추가하지 않는다. 추가 원파일2,514,056B(약2.40MiB), 실제 세 파일 합16,933,004B이며 파일당32MiB 제한으로 최악 합계는96MiB다. parsed data·atlas·시작 지연은 별도 측정 대상이다. 색상 emoji와 ZWJ 조합 지원 완료를 의미하지 않으며 실제 캡처 인수는 진행 중이다. 외부 폰트 파일을 패키지에 추가하지 않으며 내장 폰트의 license/notice는 출시 인수에 포함한다. 네트워크 폰트 로딩은 없다.

| 항목 | 원본 | dlpad 제안 |
|---|---|---|
| 본문 | 11–12px | 13 logical px, 행간 1.4 |
| 보조 | 9–10px | 12 logical px |
| 섹션/화면 제목 | 13/16px | 16/20 logical px |
| 간격 | 2,4,6,8,10,12,14,16,18,20,24,36 | 동일 scale |
| 모서리 | 4/6/8px | 동일, 기본 6 |
| 버튼·입력 | 24/32px | 기본 32, 작은 아이콘도 hit area 28 이상 |
| 상단/도구막대 | custom title44/toolbar56px | OS 제목 유지 + 기능 탐색 한 줄, 내부 브랜드 제목 제거 |
| 좌측 패널 | workspace 도구 구조 | 현재 단일 작업 화면은 compact 상단 탐색; 고정 빈 sidebar 없음 |
| 모션 | 100/200/250ms | 상태 피드백 100–200, reduced motion=0 |

본문 확대는 원본에서 의도적으로 변경하는 부분이다. OS 배율과 별도 UI 크기 선택을 제공하되 픽셀을 물리 픽셀로 고정하지 않는다. 800×600 logical px에서도 핵심 동작에 접근 가능하게 한다.

## 화면 구성

```text
┌ OS 창 제목: dlpad ──────────────────────────┐
│ 다운로드 · 설정 · 파일 목록 · 도구 · 작업 이력 │
├────────────────────────────────────────────┤
│ 화면 제목                                  │
│ 영상 URL                                   │
│ [입력                                  ]   │
│ 저장 폴더                                  │
│ [현재 경로                    ] [폴더 선택…] │
│ 품질 · 접을 수 있는 상세 설정              │
├────────────────────────────────────────────┤
│ [다운로드 시작]  작업 동작                  │
│ 현재 상태 · 사용 가능한 관측               │
└────────────────────────────────────────────┘
```

반복 카드를 쌓지 않고 구분선 있는 가상화 목록을 사용한다. 넓은 창은 선택 작업 상세를 오른쪽에, 좁은 창은 목록을 대체하는 상세 화면으로 보여주며 돌아가기를 제공한다. 상시 차트와 거대한 속도 숫자는 배제한다. 지표 상세는 상태 표시줄에서 연다.

## 컴포넌트 규칙

- 버튼: primary / secondary / ghost / danger. 화면의 주 동작만 primary. 선택된 행과 hover를 구별한다.
- 입력: 항상 label, 입력 옆 오류와 고치는 방법. placeholder로 label을 대신하지 않는다.
- 상태: 아이콘+문자+필요 시 색. `변환 중`을 `다운로드 100%`로 숨기지 않는다.
- 툴팁: hover와 keyboard focus 모두 지원. 긴 설명은 고정 가능한 도움말 패널. 필수 경고를 툴팁에만 숨기지 않는다.
- 상호작용: default/hover/focus/active/disabled/loading/error를 정의한다. disabled 옆에는 이유와 해결 동작을 제공한다.
- 그림자: 메뉴·팝오버에만 작은 그림자. 반투명 blur, 장식 gradient, 반복 애니메이션은 없다.
- 원본 CSS를 복제한 custom titlebar보다 초기에는 OS titlebar를 우선해 드래그·배율·접근성 문제를 줄인다.

## 검수

Light/Dark/System, 100/150/200% 배율, 키보드만 사용, 한글 조합 입력, 긴 제목·RTL 문자열·emoji·빈 목록·실패·10,000개 목록을 확인한다. 스크린리더가 제목/상태/버튼을 읽고 반복 진행률을 과도하게 알리지 않아야 한다. 스크린샷 비교는 OS·폰트·렌더러별 기준으로 수행한다.

## 2026-10-07 첫 번째 후보 구현 이력

기능 자동 검증을 UI/UX 인수로 표현하지 않는다. 사용자가 기존 기본 스타일·정보 구조와 아이콘 부재를 지적했으며 사용자 인수는 미통과다. CPU 후보는 ui_theme.rs에서 Light/Dark의 gtmux 역할 토큰과 치수를 적용한다. 900logical px 이상은216좌측 탐색, 그보다 좁은 창은 가로 탐색이다. 기본 다운로드는 URL·저장 위치·품질, 구간/자막과 도구/인증은 점진 공개한다. dlpad의 원 기하 아이콘은 같은 RGBA 알고리즘과 ICO 원본을 공유하며 Windows 창/PE리소스를 별도로 검증한다. [재작업과 검증 상태](docs/work-items/S00-windows-user-ui-rework.md). 새 화면의 실제 시각/배율/한글/사용자 인수는 코드 적용만으로 완료하지 않는다.

2026-10-07 실제 검증 범위: 새 Windows whole852/0/13 및 private Light→Dark→restartDark/960x640·800x600/앱 배율150·200의 실제 캡처와 한글 합성 URL readback을 검토했다. 이것은 OS DPI 변경이나 물리 IME/reader 인수가 아니다. Small/Big 창 아이콘과 PE 아이콘 추출을 확인했으나 visible taskbar pixels는 검증하지 않았다. 800폭 고급 버튼의 초기 일부 잘림, 빈 목록 안내 부재, 도구 제목/기술 설명 순서는 [후속 UX 작업](docs/work-items/S00-windows-empty-state-focus-polish.md)에 기록했다. 최종 사용자 평가·전체 가이드 상태 인수는 미완료다.

## 2026-10-07 두 번째 사용자 피드백 이후

1차 색상·sidebar 후보를 최종 gtmux 준수본으로 보지 않는다. 실제 원본의 functional toolbar·neutral hover·persistent selection·32px input/button·settings label/control 배치를 대조했다. OS 제목 표시줄의 앱 이름을 유지하고 내부 브랜드바는 제거한다. 기능 탐색은 compact 상단 막대로 통일하고 선택 상태를 지속 표시한다. 한국어 본문13/보조12px는 가독성을 위한 의도적 차이다.

저장 폴더의 label은 별도 행이다. 사용 가능한 폭360logical px 이상에서는 입력과112×32 browse를 같은 행에 둔다. 그보다 좁은 설정 카드에서는 입력이 전체 폭을 사용하고 browse를 다음 행에 둬 경로 읽기 공간을 확보한다. TextEdit inner 폭과 margin을 합산해 content16px여백을 넘지 않는다. 다운로드 browse는 draft 입력을 바꾸며 기존 실행 명령에서 저장 경로를 검증한다. 파일 목록 browse는 기존 Apply command에 연결해 위치 검증·저장 뒤 scan 결과 또는 오류를 표시한다. 취소·실패에는 기존 위치를 유지하고 로딩·빈 목록·실패 안내를 구분한다. 설정은 설명 왼쪽/조작 오른쪽이며640logical px 미만에서는 쌓는다. 네트워크 관측 상세와 기술 라이선스는 접되 필수 제품 제한 상태는 항상 표시한다.

시각 검증은 [현재 작업](docs/work-items/S00-native-folder-picker-layout.md)의 실제 바이너리·PNG로 한정한다. native 폴더 선택은 현재 Windows CPU host에 연결했으며 대체 host·다른 OS는 입력 fallback과 미지원 이유를 표시한다. OS DPI·물리IME·reader·사용자 최종 인수는 별도다.

## 미디어 작업 화면 재설계 — 2026-10-07 진행 중

현대적 UI의 기준은 장식이 아니라 작업 대상의 가시성, 다음 행동의 명료함, 상태 간 구별, 예측 가능한 입력과 복구다. GTMUX 원본의 Button/Input/IconButton/Tooltip/Toolbar2/SettingsOverlay/FileTreeView를 대조했다. 원본의 compact context toolbar, 선택된 객체 중심 작업면, 상세 설정의 점진적 노출, 독립적인 hover/selection/focus를 native 역할로 옮긴다. 모든 pressed/busy/error 상태가 원본 CSS에 이미 구현됐다고 주장하지 않는다.

흐름은 영상 준비 → 내용 확인 → 저장 구간 선택 → 저장 설정 → 다운로드 결과 확인이다. 상단은 URL과 준비 행동, 중앙은 내용/타임라인, 충분히 넓을 때 오른쪽은 저장 설정이다. 좁은 창에서는 위아래 배치한다. OS 창 제목 외 별도 브랜드 제목은 두지 않는다. 빈 화면은 입력/드롭 방법을 설명하며, 로딩과 실패는 미디어 영역에 표시한다. 실제 콘텐츠 확인 전에는 URL 문법 통과를 영상 확인 성공으로 표현하지 않는다.

공통 선택 버튼은 선택 tint/문자, hover 표면, pressed 표면, 키보드 focus outline을 구별한다. primary 버튼의 고정 fill로 hover 스타일을 덮지 않는다. 미리보기는 aspect fit/letterbox로 표시하고 늘리지 않는다. 하나의 구간 band, 시작/끝 handle, 별도 playhead를 사용하며 시간 입력과 왕복 동기화한다. 무효 수치 입력은 마지막 유효 구간을 보존하고 저장을 차단한다. 타임라인은 방향키 100ms, Shift 1초, Home/End와 접근성 numeric action을 지원한다.

새 미리보기는 320×180, 6fps 무음의 짧은 frame batch를 사용하는 개발 후보다. 전체 오디오 플레이어나 원격 사이트 지원 완료가 아니다. 로컬 영상 drop은 읽기/내용·구간 확인 범위이며 로컬 export는 후속 작업이다. 인수에는 실제 hover/down/up/선택/focus raster, 800/960 및 150/200 배율의 clipping/scroll/고정 행동 영역, 합성 영상의 실제 프레임과 오류·취소 검증이 필요하다. 현재 검증 상태는 [작업 기록](docs/work-items/S00-media-workspace-interaction.md)을 따른다.

## 2026-10-08 시각 정렬 보완

사용자가 미디어 후보의 레이아웃 일치도와 깔끔함을 재거절했다. source 영역의 중복 headline을 제거하고 URL label/32px 입력/확인/한줄 hint를 하나의 block으로 정리했다. 상단 탐색·본문·행동·상태 strip의 시작선은16px이다. 폭760px 이상에서는 작업면과300px 저장 inspector를24px 간격으로 나누고 inspector는16px 내부 여백과 역할 표면을 사용한다. 좁으면 stack한다. 품질은 동일 폭 dropdown으로 통일하고 반복 설명은 기존 도움말 안에 둔다. 선택·primary는32px, 보조label/status는12px, preview radius6, 미디어 matte와빈표면을구분한다. Timeline은32px handlehitarea를보존해72→56px로정리했다. 시작거절이유는본문에남기고하단동작에서분리했다. 실제 새 캡처/사용자평가는 별도 인수다.

## 2026-10-08 입력 내부 정렬

입력 상자 외곽32px만 맞추는 것으로 충분하지 않다. 공통 singleline 입력은 좌우12px·수직2px 여백과 가운데 수직 정렬을 함께 사용한다. 클릭 좌표·caret·selection·가로 스크롤도 동일한 TextEdit geometry를 따른다. 경로와 URL 행은 내부폭 계산에서 여백 합24px를 차감한다. focus는1.5px accent, hover는 중립 강조, 비활성 상태는 테마 기본을 유지한다. IME 시험 필드도 공통 컴포넌트를 사용한다.

관련 작업은 [입력 점검](docs/work-items/S00-input-caret-typography.md)과 [공개 영상 종단 시험](docs/work-items/S00-public-video-ui-walkthrough.md)이며, 단위 회귀를 물리 Windows 입력 검증으로 대신하지 않는다.

입력 identity도 시각 geometry와 같은 계약이다. 상위 validation/banner가 나타나거나 준비 상태가 바뀌어도 편집 중 입력의 논리 ID를 유지한다. 다운로드 옵션·URL·도구 환경·IME는 명시 ID, 폴더는 부모의 persistent ID를 사용한다. 안내 문구 삽입으로 caret/selection/focus를 잃는 autoID를 허용하지 않는다. focus 공개는 scroll_to_me(None)으로 필요한 경우만 이동시키고, 좁은 폴더 입력은 hover에서 원 전체 경로를 보여준다.

설정의 label·설명·조작이 세로로 쌓일 때는 포커스를 받은 조작만 중앙으로 옮겨 의미를 가리지 않는다. 행 전체가 viewport에 들어가면 행 전체를 필요한 만큼만 공개하고, 행이 viewport보다 크면 조작 영역을 우선 공개한다. 이 포커스 문맥 보존의 실제 회귀와 확대 GUI 인수 범위는 [현재 UI 작업](docs/work-items/S00-current-five-screen-ui-polish-20261011.md)에 기록한다.

품질 dropdown은 선택을 확정하면 닫는다. 현재 값 재선택과 키보드·접근성 선택도 같은 동작이며 pointer 클릭에만 의존하지 않는다. 열린 메뉴가 뒤의 고급 설정을 가리는 상태는 선택 완료 상태로 유지하지 않는다.

입력·재생 동작은 조건부 안내와 부모 레이아웃이 변해도 논리 ID를 유지한다. 품질 선택 popup은 현재값 재선택 및 키보드/접근성 선택에서도 닫는다. 프레임 요청 중 상태는 metadata 준비와 구분한다. 2026-10-08 실제 두 공개 URL/private GUI의 scoped 인수는 docs/work-items/S00-public-video-ui-walkthrough.md를 따른다.

네트워크 관측 설정의 단일 행 입력도 공통32px/12px/중앙 정렬과 안정적인 논리 ID를 사용한다. 인터페이스 이름 필드는 넓은 창에서도360px를 넘기지 않고 남은 폭이 좁으면 줄인다. 예시 placeholder·64자 제한·CloseWait disabled를 유지한다. 관측 적용은 실행 중 선택이며 설정 저장과 구별한다. 실제 시험 범위는 docs/work-items/S00-network-observation-input.md에 기록한다.

### 좁은 화면의 문맥과 스크롤

배율 선택으로 다음 프레임의 논리 viewport가 바뀌면 선택한 설정의 label·설명·control을 실제 새 기하로 한 번 공개한다. 키보드 focus를 강제로 이동하지 않는다. 행 전체가 viewport에 들어갈 수 없는 경우에는 control을 우선 공개한다.

하단 상태 지표는 문구 한 단위로 다음 행에 이동한다. 전체 행보다 긴 지표만 문구 내부에서 줄바꿈한다. 남은 폭 때문에 `측정 불가`의 일부만 다음 행으로 분리하지 않는다.

파일 목록의 폴더·행동·로딩/오류/빈 안내·목록은 하나의 세로 스크롤 영역에서 접근한다. 고정 탐색·하단 상태와 겹치지 않으며 목록 행은 현재 viewport에 필요한 범위만 생성한다. 작은 창과 높은 배율에서도 안내까지 실제 스크롤로 도달해야 한다. 회귀 테스트 통과만으로 시각 인수를 완료하지 않는다.

가상화 행의 widget identity는 화면에 보이는 행의 순서와 무관하게 파일 경로에 고정한다. Tab·방향키로 새 행에 포커스가 이동하면 해당 행을 한 번 공개한다. 같은 행의 포커스가 유지된 상태에서 사용자가 스크롤한 화면을 매 프레임 원위치로 돌리지 않는다. 키보드 탐색과 선택·미리보기 실행을 구분하며 Enter에서만 기존 미리보기 요청을 사용한다.

## 다운로드 화면의 빈 미리보기와 간격

준비 중이거나 준비된 영상이 있는 미리보기는 기존 aspect-fit 및140–200 logical px 영역을 유지한다. 준비되지 않은 idle 상태는 아이콘·설명을 위한96–120 logical px 최소 영역을 사용하며, 긴 오류 전문이 필요하면 영역을 자연스럽게 확장한다. 좁은 창에서 미리보기와 저장 설정 사이의 중복 여백을 제거하되32px 조작 영역과 필요한 local/range/preview-bypass 설명 간격을 유지한다. yt-dlp 다운로드 조건과 FFmpeg 미리보기 조건은 서로 달라 무조건 같은 오류로 합치거나 숨기지 않는다. 실제 Light/Dark·800/960 캡처 검증 전에는 이 변경을 시각 인수 완료로 기록하지 않는다.

## 미지원 관측 설정

관측 counter가 없는 OS는0으로표시하거나효과없는인터페이스적용을활성화하지않는다. 이전 Windows 후보는 미지원·비활성으로 표시했다. 2026-10-12 worker 소유 counter와 Unicode 수동 선택을 연결해 전체 검사를 통과했으며 자동 선택과 실제 제품 화면 인수는 진행 중이다. 미지원 OS는 비활성 이유·작업 다운로드 속도와의 차이·기존 선택 보존을 설명한다. Linux지원설정의입력/유효성검증은유지한다. disabled상태를focus/typing검증성공으로표현하지않는다. 설명의실제가시성과스크롤은별도native캡처로검수한다.

## 2026-10-12 저장 설정과 도구 회복 개선 — 검증 중

넓은 다운로드 화면의 저장 inspector는400logical px로 조정했다. 내부368px에서 저장 경로 입력과112px 폴더 선택을 같은 행에 배치하며, 좁은 화면은 기존 stack을 유지한다. 실제 준비된 영상의 크기·clip·footer 겹침과 확대 화면 인수는 아직 검증 중이다. 이전300px 카드 기록은 구현 이력이다.

누락된 yt-dlp·FFmpeg의 짧은 사유와 필요한 도구 이름을 명시한 설치 화면 이동 버튼을 같은 wrapped 행으로 제공한다. 입력 초안을 보존하고 기존 도구 화면으로 이동한다. 사유와 회복 동작이 모두 없으면 빈 행을 만들지 않는다. URL 미리보기 오류의 동일 사유만 중복 제거하며 FFmpeg와 다운로드 도구의 역할은 구별한다. 실제 클릭·키보드 Context 회귀는 통과했으나 최신 native raster 및 사용자 UI/UX 인수는 별도다.

네트워크 상세를 새로 펼칠 때는 미지원 안내를 비활성 입력보다 먼저 표시하고, 애니메이션 종료 후 안내 영역을 한 번 공개한다. 같은 frame의 재pass에서는 요청을 유지하되 이후 사용자 스크롤에는 재공개하지 않는다. 실제 본문 paint clip과 확대 screenshot 검증 전에는 공개 완료로 판정하지 않는다. 지원하지 않는 플랫폼에서도 저장된 인터페이스 값은 그대로 유지한다.

탐색 선택 버튼의 눌림은 hover와 구별한다. 비선택 hover는 Light232/Dark60 중립 배경, pressed는 Light212/Dark80을 쓴다. 선택된 pressed는 기존 선택 배경을 유지하며 hover 강조와 구별된다. 마우스 활성화와 Tab 키보드 포커스를 따로 검증한다. 준비된 영상 canvas는 일반140–200px에서 실제 body 높이−8px까지 축소해 확대 clip 경계에 여유를 두며,160×90 이상 이미지 인수 조건을 검증했다. native 시각 인수는 별도이다.

체크박스는 14px indicator에 radius3을 한정해 라디오 버튼과 시각적으로 구별한다. 일반 버튼 radius6은 유지하며 native Checkbox 의미·checked·disabled·Tab focus를 보존한다. 도구 화면은 recovery 안내부터 상세·설치·목록까지 단일 본문 스크롤을 사용한다. 설치 버튼 포커스 획득 또는 같은 포커스를 유지하는 도구 탭 변경 순간에 한 번 공개하고, 이후 매 frame 스크롤을 강제하지 않는다. 실제 확대 캡처의 footer 가림은 provider bounds 통과와 별개로 실패로 기록한다.

## 첫 실행 기본값

저장값과 환경 override가 없으면 복구 worker가 OS 문서 위치의 dlpad 경로를 resolve해 저장 폴더 입력에 자동 표시한다. 첫 화면 표시만으로 media 폴더를 만들거나 저장값을 덮어쓰지 않는다. 최고 품질·전체 영상·자막 선택 없음·고급 도구 기본 정책이 초기값이다. 사용자는 원하는 항목만 변경한다. 유효하지 않은 기존 경로는 기본값으로 조용히 대체하지 않고 복구 가능한 오류를 표시한다. 실제 Windows 첫 실행 화면 검증은 진행 중이다.

## 2026-10-12 고급 속도 입력 — 검증 중

속도 제한은 제한 없음·1/5/10 MB/s preset과 숫자/단위 조작을 제공한다. MB/s를 기본으로 SI 단위 정수 환산을 사용하고 무효 draft에서 Start를 차단한다. 일반 설명과 기본값은 짧게, CLI B/s 예시·영향·충돌은 상세 안에 둔다. Windows actual Form 회귀30개는 통과했으며 확대 GUI·단위 popup·실제 화면 인수는 남았다.

## 2026-10-15 도움말 표현의 일치

설명만 있는 별도 줄이나 떠 있는 물음표를 만들지 않는다. 선택 설명은 관련 라벨 옆의16px 벡터 정보 아이콘으로 연결하며32px 클릭 영역은 유지한다. 기본 상태는 muted이고 hover·pressed·pinned·focus·disabled를 구분한다. 키보드로 읽거나 popup을 고정할 수 있으며 Escape는 원 도움말로 돌아가고 밖 클릭은 새 대상의 focus를 보존한다. 필수 오류·인증 보존 안내를 도움말로 숨기지 않는다. [구현과 현재 인수](docs/work-items/S00-inline-help-info-icon-20261015.md).

## 2026-10-15 결과 공개와 도움말 문맥

검색 완료 안내는 최신 요청의 첫 성공에 한 번만 드러내며 입력/selection focus는 유지한다. draft 편집·실패·Stop·늦은 결과를 새 결과 공개로 표현하지 않는다. 이력 header/search/paging/records는 하나의 body scroll에 두어 작은 화면에서도 결과로 이동할 수 있게 한다.

Escape는 고정 도움말 제목과 본문을 모두 닫고 owner focus를 유지한다. 동일 연속 focus/hover에서는 다시 열지 않으며 새 클릭/포커스 또는 hover 재진입에 재허용한다. 툴팁·focus 계산은 Context 데이터 write lock 밖에서 읽는다.

다운로드 도구 경로의 focus 전이에서는 label/info/input/browse 실제 rect의 전체가 viewport에 들어갈 크기이면 그룹을 드러낸다. 크면 focused control을 우선한다. held focus나 이후 wheel/다른 입력으로 이동할 때 매 프레임 다시 맞추지 않는다. 이 변경의 actual Context 검증은 통과했으며 새 PE의 native/pixel 검수는 후속이다.


작업 이력의 상태는 완료/실패/취소됨/다운로드 중처럼 짧게 표시하고 보존 결과·재개 불가 이유는 본문에 유지한다. 검색 결과 자동 공개는 실제 ID Label의 rect/접근성 역할을 변형하지 않는다. 전체 행과 필수 안내가 body clip에 들어갈 때만 그 그룹을 한 번 드러내며, 넘치면 원 ID를 우선한다. 현재 격리 Context12개·strict clippy는 통과했고 제품 통합/새 화면 인수는 [현재 작업](docs/work-items/S03-history-status-compact-20261015.md)에서 구분한다.

## 2026-10-16 영상 열기·진행 상태·작은 도움말

로컬 영상 상세는 제목 아래에 기존32px secondary 버튼으로 기본 앱 열기와 다른 앱 선택을 둔다. 기본 열기와 선택의 차이·일시disabled 이유는 hover 도움말에서 설명하며 진행 중에는 중복 요청을 막고 짧은 상태를 표시한다. 사용자 소유 외부창을 앱 내부 재생처럼 표시하지 않는다. 원격 URL에는 로컬 파일이 필요한 이유를 구분한다.

도구 확인을 설치로 오인하지 않도록 본문과 하단은 실제 작업 종류에 맞는 같은 라벨을 사용한다. 기존cache 목록은 유지하며 실행 직전 재검증과 취소·종료gate는 유지한다. 일반 조회에 설치120초 안내를 상시 노출하지 않는다.

긴 보호 도움말은 입력/info/browse union 위·아래의 실제 여유에서 popup frame·shadow·제목 비용을 빼 scroll 상한을 정한다. 읽을 최소 높이가 없는 극소창은 기존fallback임을 구분한다. actual Context RED재현/GREEN·기존회귀는 통과했고 실제currentPE popup frame/끝문장은 후속검수다. UIA raw Text bounds를 clipped viewport로 오인하지 않는다. [현재 통합](docs/work-items/S00-ui31-integration-20261016.md).
