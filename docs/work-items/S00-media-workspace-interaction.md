# S00: 미디어 중심 작업 화면과 상호작용 재설계

- 상태: in-progress
- 분류: feature
- 우선순위: P0 (사용자 추가 범위)
- 요구사항: R01, R04, R05, R08, R11
- 담당: root 통합/화면/구간 선택/카탈로그/모듈 선언/Cargo/최종 검증; diagnostic_cache GTMUX 전체 상호작용 감사(/tmp); metrics_recovery 미디어 service 초안(/tmp); soak_recovery_audit hover/selection/scroll 검증 초안(/tmp)
- 수정 파일: root crates/app/src/accessible.rs, ui_theme.rs, catalog.rs, accessible_cpu_main.rs, accessible_main.rs, software_host.rs, 새 app/ui_components.rs, media_workspace.rs, media_preview_worker.rs, media_timeline.rs, core/media_selection.rs, engine/media_preview.rs 및 각 lib.rs; DESIGN.md, CONTEXT.md, 요구사항/ADR, 검증 및 사용자 안내; scripts/windows_media_url_private_probe.cs, scripts/windows_media_workspace_layout_private_probe.cs, scripts/windows_library_thread_window_cost60_probe.cs. 하위는 프로젝트 직접 수정 없음.
- 의존성: 두 번째 UI 후보와 사용자 재거절; 기존 process owned-tree/cancel/timeout, tool 선택, native CPU texture host.

## 사용자 결과

URL 또는 드롭한 영상에서 미디어 내용을 확인하고 시각적 구간을 선택한다. 전체 화면의 hover/focus/pressed/selected/disabled/loading/error와 스크롤/반응형 배치를 일관되게 만든다. GTMUX의 원 컴포넌트와 화면 구조를 조사해 표면 색 복사에 그치지 않는다.

## 계약

Rust native 유지, blocking I/O/도구 실행 UI thread 금지. Preview는 다운로드/파일 수정/인증 전송 권한이 아니다. URL 문법과 실제 콘텐츠 확인을 구별한다. 채널/프레임/metadata/time 상한을 명시하며(원격 전체 수신 byte와 RSS hard cap 미확보), 오래된 요청 결과를 폐기한다. 합성 영상과 소유 TEMP만 검증한다. Git 없음 보존, native/Cargo 실행 root 직렬; 같은 파일 동시 수정 없음.

## 인수 조건

실제 GTMUX button/input/tabs/toolbar/scroll/empty/loading/error와 한국어 화면을 대조한다. 정상/hover/pressed/selection/focus/disabled PNG와 pointer/keyboard 조작,800/960/150/200 배율에서 content/scroll/footer 겹침 부재를 확인한다. 유효 URL 준비/실패/취소/늦은 응답, 로컬 영상 drop/미지원 거절/미리보기, 구간 handles/keyboard/시간 입력 동기화와 invalid 구간을 시험한다. Windows whole/실제 private GUI/새 비용/새 ZIP/문서 검사 후 scope와 미검증을 구분한다.

## 사용자 최신 방향

개별 지적의 체크리스트만 처리하지 않는다. 미디어 준비→미리보기→구간 선택→저장→결과 확인 흐름, 정보 위계·밀도·상태 피드백·학습 비용·키보드 접근을 함께 설계한다. 이전63PNG/878test는 기능/정적 scoped 증거이며 UX 인수 통과가 아니다.

## 현재 통합 및 검증

GT­MUX 원 Button/Input/IconButton/Tooltip/Toolbar2/SettingsOverlay/FileTreeView와 보관 화면을 병렬로 읽기 전용 감사했다. root가 공통 상태 버튼, 미디어 서비스/최신 요청 worker, core 선택 불변 조건, aspect-fit 미리보기, 시각 타임라인, 수치 구간 왕복, wide/narrow 작업면을 통합했다. 기존 폴더 선택을 재사용하고 dependency/Cargo.lock은 변경하지 않았다. 하위 에이전트는 프로젝트 파일/native 실행을 하지 않았다.

초기 Windows collector는 sandbox WSL interop socket 실패로 실제 Cargo를 실행하지 못했다. 허용된 native 검증만 sandbox 밖의 기존 private desktop owned host로 실행했다. 첫 Clippy에서 frame multiple 검사와 PathBuf 참조 lint, 다음 실행에서 timeline UI 인수 및 test module 위치 컴파일 오류를 발견해 수정했다. 실패 로그는 삭제하지 않았다. 전체 검증과 실제 GUI 인수 전이며 완료 상태로 변경하지 않는다.

미검증/후속: 새 코드의 whole 성공, 실제 pointer/keyboard/scroll/배율과 합성 영상 준비·seek·재생·취소·close, 새 idle/preview 비용, 최신 ZIP 생성. 로컬 export/오디오 플레이어, 모든 원격 사이트/인증 필요 preview, physical IME/reader/OS DPI 및 전체 P0 릴리스는 이 작업의 통과로 대신하지 않는다.

## 2026-10-07 실측 갱신

현재 stable Windows whole의 fmt/clippy/test/release가 모두 통과했다. 합성 2초 빨강→파랑 원본으로 engine 준비/프레임 batch/끝 지점 seek/취소·deadline·원본 보존과 실제 Candidate RawInput hover/down/up/Tab/drop/재생 시간·색상 변화/CloseWait ACK를 통과했다. 증거는 `docs/validation/evidence/windows-msvc-media-workspace-stable-20261007/`와 `windows-msvc-media-synthetic-stable-20261007/`이며 19 PNG를 보존했다. root가 frame-12/16/17의 한국어·빨강/파랑·일시 정지·종료 disabled 상태를 직접 열었다. 후처리 로그가 Cargo test 이름 뒤에 붙어 anchored regex가 실패했으며 실제 테스트는 성공했다. 수집 파싱만 바로잡아 PNG를 복구했다. 실제 HWND mouse route·배율/스크롤 시험은 현재 진행 중이며 RawInput 증거로 대체하지 않는다.

추가 holistic 감사: 긴 영상의 정밀 확대/겹친 handle 접근, 자동 확인과 명시 버튼의 의미, 파일 목록→미리보기→저장→결과의 일관성이 후속 개선 우선이다. 색상 변경만으로 현대 UX 완료를 주장하지 않는다.

실제 Windows layout-uia 검증: stable 원 바이너리에서 두 process/Light→Dark→restartDark,800/960 및150/200 앱 배율의63PNG와 저장 폴더 행32px/margin/scroll·focus 도달을 확인했다. 원 process natural0/Job0/private desktop close/input unchanged. 선택 Button은 UIA TogglePattern으로 노출되어 observer를 조정했다. 별도 WM_MOUSEMOVE/down/up 시험은 private desktop에서 selection 전환을 검증하지 못해 FAIL을 보존했다(visible desktop fallback 없음). RawInput과 UIA를 실제 물리 마우스 인수로 승격하지 않는다. root가 selected800/960/150 다운로드와200설정·800목록 PNG를 열었다.

마지막 소규모 보완: disabled 상태 timeline keyboard/AccessKit 변경을 거절하고, focus 획득시 ancestor scroll 및 변경후 repaint를 요청한다. URL 자동 확인 동작을 catalog의 안내에 명시했다. 새 실패 경로 회귀를 추가했으며 현재 focus-safe whole 재검증 중이다. .git은 git status에서 정상 저장소로 인식되지 않으며 재구성·삭제하지 않았다.

최종 focus-safe whole905/0/16, source608,4검사exit0을 통과했다. 최종 합성19PNG도 재실행해 통과했다. URL 실제 GUI는 `windows-msvc-media-url-tool-cap-20261007/`에 보존했다: 합성 metadata helper·127.0.0.1 한정 서버·real FFmpeg HTTP Range 2건4516bytes·red→blue·UIA play·자연 종료 및 원본SHA가 통과했다. root가 실제 blue/활성 다운로드 화면을 열었다. 실제 yt-dlp/site extractor·physical mouse는 이 증거에 포함하지 않는다. 최초 외부 FFmpeg SHA helper의128MiB상한 실패를 기록하고, 확인된129MiB도구 두 개에만256MiB streaming SHA상한을 적용했다. 최종 HWND layout 재실행 중이며 새 비용/ZIP이 남는다.

최신 final layout63PNG/2process도 current exact source와 바인딩하여 통과했다. root가 첫960와 재시작200 다운로드 화면을 열었고 독립 검토 report를 evidence에 보존했다. 200% 고정 하단은 보조 버튼 때문에 두 줄을 차지하며 향후 문맥에 맞춰 간소화할 UX 한계다. 최신 cost는 첫60초/6구간을 통과했고 나머지 round 측정 중이다.

성능 후속: cost-final18구간 중2개CPU1.098700/1.103177%로실패했다(자료보존). 별도 `S00-windows-restore-idle-event-wait.md`에서 Windows 복귀20ms유휴poll을 실제red4wait→green1wait로교정했고 새whole906/0/16을통과했다. 이새binary에대한cost와packaging을다시바인딩한다. 이전905GUI/URL/PNG는이력으로보존하며소스전체동일의최종근거로승격하지않는다.

최종 통합: event-idle whole906/0/16, 새 비용18창 전부 통과(최대CPU0.936669876%/WS69.97MiB), 현재binary의실제private URL GUI/realFFmpeg Range2/4516B/red→blue/원본보존/자연종료를root·독립검토했다. URL은합성extractor이고실제사이트추출검증이아니다. final63PNG와RawInput19PNG는905artifact이며현재906과608source를대조할때차이는engine/activation/windows.rs한파일이다. UI/core구간/enginepreview소스동일성은composite-review.json에기록했다. 물리pointer/OLEdrop·IME/reader/OSDPI·장기정밀구간·200%footer·library연결·최종사용자UI인수는남는다. [GTMUX 실제소스비교와현대UI판단](../development/modern-ui-audit.md). 전체P0완료로보고하지않는다.

최신개발ZIP: windows-msvc-media-workspace-graph-event-idle-20261007/dlpad-user-preview.zip; SHA2569d5187543f27524a280c883e99f4c6a8623f2703519533d317999da7b35a539e/13,842,744B/466members. root가CRC·member경로·timestamp·모든binary와whole/graph/lock pins를별도검토했다. 공개release/서명/정식installer완료는false이며선택graph공지258/missing0,wholelockmissing43을보존한다.

독립ZIP검토도동일SHA/466members/CRC/3binary PE역할/whole906·graph·lock/공지258missing0으로통과했다(package-independent-review.json). 이작업은구현·검증된개발후보를제공한상태이며최종UX인수와별도후속때문에in-progress를유지한다.
