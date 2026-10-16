# S11: 긴 경로 설치 앱의 초기 종료 진단

- 상태: in-progress
- 분류: bug-investigation
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: installer_batch — 독립 native helper; advanced_units — vendor/winit 후보; root — 실행·공유 계약·소스 통합·문서
- 선행 작업: v21 whole/package/native normal 인수

## 사용자 결과

한글 긴 경로에 설치된 앱도 초기 OS 드래그 등록 오류로 종료하지 않고 파일 입력과 정상 종료를 제공해야 한다.

## 계약과 범위

원 v21/577·598 UTF-16 임시 설치를 보존한다. 사용자 폴더·Desktop·레지스트리·보안 설정을 변경하지 않는다. 새 소유 private Job/Desktop의 진단 재현은 설치기 실행 성공과 구분한다. preferred OLE/native drop fallback/COM 수명은 Root가 조정한다.

## 인수 조건

실제 설치→검증→앱 HWND→WM_CLOSE→앱·감독 자연0/Job0, 원20객체·6부모·생성7파일·journal 보존. 초기 오류는 원 HANDLE/PID/creation·exit와 bounded stderr로 판독한다. 단위 회귀를 물리 드래그 인수로 대신하지 않는다.

## 검증과 handoff

Child freshhelperonly,Root가 소스·네이티브 실행·문서·증거 통합 담당. 원v4577/598설치Verify가actual로진행했지만AppIdentitylive실패·forcedcleanup. 변경파일observer.cs+freshdriveroutput문구,collector v4byte동일. 생성완료앱의원Job발견PID/HANDLE/creation/exit/wait/job/status단계관측을4실제callsite전에추가한다. ≤16rows/status1024/기존exception16KiB/전체JSON262144B. 원 검증 조건과 시간 상한 유지. Default desktop·OS 설정 변경 없음.

인수:Root actualstack+PID/creationquery/wait/exit·supervisorstatus/jobtrace로runtime종료와조회/수명Race분리,그뒤근거있는제품또는fixture회귀. 본준비단계는컴파일/실행없고원인확정/앱실행성공/실제GUIready가아님. 기존 실패와 시험 원본 파일 보존.

Root 통합·검증은 이 기록과 원자료를 대조하며 실제 미실행 항목을 PASS로 표시하지 않는다. helper candidate와 제품 source는 별개다.

실제v5실행: 577 UTF-16 target/598 exe의설치·검증·stage재검증이성공한뒤originalchildPID6380/creation134363860411746203/HANDLE이처음259/Job2를보였다가약0.76초뒤exit101/Wait0/Jobactive1을기록했다. visibleAppHWND는관측못했다. 이는source가설과구분되는실제earlyexit이며parent259와같이기록했다. 원FAIL/forcedownedcleanup은보존한다. stderr를같은소유exe/새단기fixture/privateJob로수집하는독립repro를준비중이고installerRunPASS로대체하지않는다.

Root 독립 stderr 재현 v1은 앱 시작 전 generic host의 Win32Error3/result125/소유 강제 정리로 실패했고 원자료를 보존했다. v2는 동일한 소유 파일의 Win32 extended-length 경로를 전달하여 private Job에서 앱 exit101을 재현했다. 원 로그는 vendor/winit/src/platform_impl/windows/window.rs1194의 RegisterDragDrop assertion 실패(-2147352577=0x8001ffff, 기대 S_OK0)를 보여준다. 정상 Job0·private desktop 종료·입력 데스크톱 불변과 원 source/20객체·6부모 namespace/전체 byte 보존은 Root가 독립 대조한다. 장경로와 Windows OLE 오류의 내부 인과 및 실제 사용자 desktop의 동등성은 미확정이다. OLE 성공 경로를 보존하고 등록 실패 시 bounded native WM_DROPFILES로 동작을 유지하는 vendor 후보를 준비한다. 이 재현은 설치기 Run 성공 증거를 대신하지 않는다.


2026-10-14 Root 후보 통합: `tool-isolated-source`의 vendor/winit 7파일에 소스 핀을 대조한 뒤 적용했다. Root 검토에서 OLE decoder가 `Some(())`를 반환하면서 반환형은 이전 `Option<HDROP>`를 유지한 컴파일 불일치를 찾아 `Option<()>`로 교정했다. Windows host는 숨김 창 생성 직후 등록 결과를 한 번 받아 별도 세션 안내로 표시한다. 오류·복구·다운로드 진행 상태를 덮어쓰지 않는다. 신규 crate/version/feature/lock 변경은 이 드래그 패치에 없다. 이 변경은 아직 제품 소스/v21 ZIP에 채택하지 않았다.

첫 native 실행은 WSL interop `UtilBindVsockAnyPort socket failed 1`로 앱 컴파일 전에 실패했다. 원 결과를 보존하며 테스트 실패/통과로 승격하지 않는다. 사용자가 허용한 네이티브 실행 범위의 권한으로 같은 격리 소스를 새 출력에 재실행 중이다. 시스템 설정·WSL 재시작은 수행하지 않았다. 원 전체/최신 빌드·실제 긴 경로 앱 HWND/종료·정책 단위 테스트가 남는다.


후속 compile-v2는 기존 `WindowData`의 공유 참조에서 Option을 변경할 수 없는 실제 E0596으로 실패했다. 원 Windows 이벤트 callback 참조 계약을 변경하지 않고 RefCell에서 registration을 먼저 take한 뒤 borrow를 해제하고 deactivate하는 방식으로 교정했다. OLE revoke 중 재진입이 가능한 상태에서 RefCell borrow를 유지하지 않는다. compile-v3/v4는 CPU 회귀1개는 통과했으나 공유 Candidate를 쓰는 대체 eframe 타깃의 미사용 setter로 strict 검사에 실패했다. CPU host 전용 getter 연결과 대체 host의 창 소유 차이를 제한적인 dead_code 허용으로 명시하고 v5 앱 전체·strict 검사로 재시험한다. 해당 실패를 성공 로그로 덮지 않는다.

원 vendor 전체 단위 시험 bridge-v1은 optional Linux sctk-adwaita의 crossfont 오프라인 공급 누락으로 setup 실패했다. Windows 구현의 실패/통과가 아니다. 기존7시험과 SDK0.52 구현을 유지하는 별도 Windows 의존성 manifest bridge를 준비하며 제품/root 잠금파일을 바꾸지 않는다.

후속 Root 통합: 원 full winit 구현과 SDK0.52를 유지한 Windows 전용 isolated manifest의 실제7정책·스타일 회귀를 실행해 통과했고, 원 소스 byte-identical/lock checksum·native raw/source/privateJob을 독립 대조했다. 제품 현재source669에 드래그 후보를 채택했고 새전체22 검증 중이다. 원 긴 경로 HWND/종료와 실제 WM_DROPFILES 네 경우 시험은 다음 인수이며 현재 성공으로 보고하지 않는다.

최신 v22 제품에서 인수 조건의 긴 경로 복구를 실제 확인했다. target577/exe598 UTF-16, 앱 실제private HWND 및 앱·감독 자연0/Job0, 원20객체6부모·5payload5journal/생성7파일 보존을 Root가 독립 대조했다. 현재 full Windows네명령1,524/0/42, 별도 vendor 정책7·actualWM_DROPFILES4case도 통과했다. 긴 경로44PNG를 모두 직접 팀 검수하고 Root는44원SHA/크기/해상도·중요2원이미지를 재확인했다. OLE오류의 Windows내부인과·실제사용자desktop·물리DnD 인수는 별도이며, 원v21panic와collector가정오류는 보존한다. [최신 실제 실행](../validation/evidence/windows-msvc-ui-feedback-v22-20261014-long-install-run-postguard-v1/root-review.json), [44화면](../validation/evidence/windows-msvc-ui-feedback-v22-20261014-long-install-run-postguard-v1/root-team-pixel-review.json).
