# S00: Windows 최초 가시 프레임 테마

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06
- 담당/수정 범위: root는 이 문서·공유 인터페이스·후속 통합 및 실제 검증, diagnostic_cache는 /tmp/windows_first_paint_theme_plan.md 읽기 감사 초안만 담당한다. 제품 source/Cargo/lock은 root만 수정한다.
- 선행 작업: 현재 Windows CPU 전체검증과 격리native observer/원ownedfixture 계약. 현재동시실행검증의source는동결한다.

## 범위와 인수

software_host.rs와 private observer의 원firstvisibility/최초clientpixel·저장Light/Dark/System을 실제검증한다. 시스템theme설정과사용자desktop을변경하지않고원savedsetting루트만사용한다. 계획/소스분석을actualfirstpaint로대체하지않는다.

읽기감사후구체적인최소단위로구현/실행하고root가원로그·파일SHA·프로세스정리를독립검증한다. 아직actual후속인수완료가아니다.

2026-10-05 명시 재개: diagnostic_cache 담당은 새 private observer/runner를 /tmp/dlpad-first-theme-*에 구현한다. root가 검토 후 프로젝트 통합·원native실행·최종검증을 맡는다. 공유Rust/Cargo/lock/nativeprobe 실행은 root 승인 전 금지한다. 독립license감사는 soak_recovery_audit가 /tmp/dlpad-license-next-*만 담당한다.

2026-10-05 첫관측collector는원EnumDesktopWindows false/no-window0과Winit Thread Event Target 보조창을실제관측했다. 원실패후exactdlpad/Window Class 필터와boundederror/count기록으로Dark/Light/System3회수집·sourcepins·자연exit0/Job0·desktop원복이통과했다. [Dark 원검토](../validation/evidence/windows-first-observed-theme-dark-enum-fixed-20261005-resume/root-review.json), [Light 원검토](../validation/evidence/windows-first-observed-theme-light-enum-fixed-20261005-resume/root-review.json), [System 원검토](../validation/evidence/windows-first-observed-theme-system-enum-fixed-20261005-resume/root-review.json). System의초기GDItile은terminal/stale처럼보였다가Light앱으로변경되어actualappbuffertheme와동일성미확인이다. 이결과로wrong-theme버그나최초presentationPASS를확정하지않는다. root는소유renderbuffer/ctx/windowtheme의첫successfulpresent receipt와initialhidden→present→show순서를다음계약으로검증한다. 별도feature first-paint-fixture는기본OFF/기존windows-sys재사용이며Cargo/hostentry/root만수정한다. diagnostic_cache는/tmp새receiptmodule·observer초안만담당한다.

2026-10-05 구현범위확정: root는 software_host.rs의initial_show_pending 상태로 AccessKit 초기화후hidden actualframe.present 성공시에만최초show를수행한다. 실패/zero-size는숨긴상태에서오류종료하며무조건showfallback없다. Cargo first-paint-fixture defaultOFF와 first_paint_fixture.rs/CPUentry 및 새scripts/windows_first_paint_private_probe.py/.cs는원3cap의첫ownerreceipt만전달한다. 계측에텍스트/경로/비밀데이터없으며추가dependency/version없고binarydelta는actualbuild후기록한다. 정상default제품동작과계측원실증·Systemctx/nativeTheme일치·원PNG/Jobcleanup·전체fmt/clippy/test/release가인수조건이다. fixture전문자료와rawGetDC만으로compositor scanout/OS설정실제변경전체를증명하지않는다.

최신계측전체fmt/clippy/test/release4명령은actualWindowsGNU PASS이며원cap5회귀도PASS다. 최초hostargvLinuxCargo경로오류·ValidationError변환3·Clippy4경고는원실패를보존하고각각교정했다. [전체검토](../validation/evidence/windows-first-paint-feature-checks-clippy-fixed-20261005-resume/root-review.json). actualowner첫present3case는hiddenbefore/afterpresent=false→show=true/sequence1/원PID-TID-HWND가PASS: Dark실제buffer/ctx27RGB,Light248RGB,System원nativeLight와ctx/실제buffer248RGB일치. [Dark](../validation/evidence/windows-first-paint-owner-dark-20261005-resume/root-review.json), [Light](../validation/evidence/windows-first-paint-owner-light-20261005-resume/root-review.json), [System](../validation/evidence/windows-first-paint-owner-system-20261005-resume/root-review.json). 자연exit0/Job0/private원복을root독립확인했다. 이후FFmpegfixture진단source변경은현재host미변경이며최신전체근거는default복원후다시확인한다. 기본OFF제품복원/가시테마확인은아직진행중. compositor scanout/실제OSTheme변경은완료로쓰지않는다.

최신기본defaultOFF CPU 전체4필수명령과root current535소스SHA 검증을완료했다. [기본복원원검토](../validation/evidence/windows-published-full-current-default-20261005-resume/root-source-review.json). 기존원owner3테마·firstpresent순서와현재동일software_host/first_paint_fixture소스를구분검토했으며전체S00·compositor scanout·실제OS System 변경·reader 채택을완료로확대하지않는다. 현재CPU7871488B는기존보고서cap기본7856640B대비14848B 증가이나notice schema 통합도포함해 firstpaint 단독증분으로표현하지않는다.
