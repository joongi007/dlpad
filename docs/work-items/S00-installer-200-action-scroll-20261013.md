# S00: 설치기 200% 하단 명령 스크롤 검수

- 상태: completed (scoped)
- 분류: UX
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: root — 실행/원본 대조/직접 시각검수/본 기록; installer_batch — /tmp C# fixture/collector 후보만; 제품 소스 변경 없음
- 선행 작업: v15 전체 Windows1,307/0/33, 별도privateGDI3/0/0, 최신 설치·실행direct0·변조거부·63PNG 통합

## 사용자 결과

설치 화면을200%로 확대해도 스크롤로 하단 설치/재검증/실행 명령에 도달하고 글자·조작을 온전히 볼 수 있다.

## 계약과 범위

현재200% top 캡처에는 하단 명령이 scroll 밖에 있다. 이를 누락/도달불가로 판단하지 않고 owned HWND의 실제 WM_VSCROLL/SB_BOTTOM 처리 후5command의현재ID/부모/client bounds/이름/enabled 및 실제PNG를 확인한다. SB_TOP 복귀와 원테마·배율 복원을 유지한다. 사용자 Default desktop/globalinput/clipboard/focus·실제 OS DPI/HC 설정 변경 금지. 기존C# native0/Job0/MTA·원7source/5payload/journal/currentwhole/current3PE·stagecopy/fontclone scope·메모리/로그/시간caps 보존. geometry 성공을 물리focus/pixel성공으로 대신하지 않는다.

## 인수 조건

실제currentv15 Dark200의 하단5명령이 body안에 들어오고 verified Run·Verify 상태가 유지되는지 검증한다. exactPNG를 root가 직접 열어 한글·간격·잘림·배치를 확인한다. 원top캡처/원FAIL을 보존한다. 새사실이 실패이면 제품 원인과observer조건 실패를 구분한 뒤 범위안수정을 진행한다. 전체GUI/physicalhover/OSfocus를 선언하지 않는다.

## 검증과 handoff

/tmp 추가phase 후보 준비 중이며 아직 실제scroll시험을 실행하지 않았다. 제품src/Cargo/schema 변경 없이 최신 설치payload를 재사용한다. 기존nativepicture검수에서top200화면의action영역은아래에있음을 root와하위가동일확인했으며 이것만으로 결함을 주장하지 않는다.


실제compile/native/directcollector outer0 및원normal조건PASS. Root는원source/current3PE/7source/5payload/journal·자연app/supervisor0/Job0를독립대조했다. Dark200owned SB_BOTTOM 후5command201/202/203/204/212 현재PID/thread/creation/class/runtimeID/name·높이64·전체경계viewport927×961내부 및상태(Install/Stopdisabled,Verify/Remove/Runenabled)를확인했다. topheading은y-315..-251로scroll밖임을분리한다. exactPNG를root가직접열어5개한글명령/전체조작·간격/배치가보이고잘림없음을확인했다. SB_TOP/theme/scale원복과이후설치수명검사도통과했다. physicalmousewheel/tab/actualOSDPI/reader는별도이며전체UI인수아니다. [원native/독립대조](../validation/evidence/windows-msvc-ui-cross-surface-v15-20261013-dark200-scroll/root-review.json), [직접PNG검수](../validation/evidence/windows-msvc-ui-cross-surface-v15-20261013-dark200-scroll/root-scroll-visual-review.json).
