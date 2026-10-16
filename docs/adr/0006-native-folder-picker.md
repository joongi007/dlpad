# 0006: Windows 네이티브 폴더 선택과 원 창 수명

- 상태: Accepted — Windows CPU 후보의 scoped 구현; 전체 S00/릴리스 스택 선정과 별도
- 날짜: 2026-10-07
- 요구사항: R01, R08, R11

## 문제

사용자는 저장 위치를 운영체제 폴더 선택 창에서 고를 수 있어야 한다. Rust 네이티브 UI와 긴 I/O의 UI thread 분리를 유지하고, COM Show의 중첩 메시지 루프가 Rust UI borrow를 재진입하지 않도록 한다.

## 결정

기존 Windows API dependency를 재사용하며 WebView·서버·새 crate를 추가하지 않는다. Windows CPU host는 원 Window의 Arc를 전달하고 transient STA worker가 IFileOpenDialog와 hidden owned proxy를 만든다. 소유 연결은 dialog→proxy→원 app HWND다. UI는 고정 크기 completion channel과 비차단 취소 메시지만 처리한다. proxy 원 PID/TID·nonce와 HWND 수명을 검사하며 COM close/release를 Mutex/TLS borrow 바깥에서 수행한다.

취소는 결과 적용보다 우선한다. 이미 선택값이 completion에 있어도 이후 UI cancel 요청이 있으면 선택을 폐기한다. 정상 앱 종료 및 fatal host drain은 원 dialog의 완료 ACK까지 event loop와 원 창을 유지한다. ACK는 worker 정리 시점이며 OS cleanup 실패를 정상 cleanup으로 보고하지 않는다.

다운로드 browse는 draft 경로만 갱신한다. 파일 목록 browse는 기존 Apply command를 호출한다. 위치 검증·저장과 scan은 기존 worker/engine 정책을 따른다. native cancel·picker 오류·위치 검증 실패는 기존 위치를 보존하며, 검증된 새 위치의 scan 실패는 새 위치와 오류를 표시하는 기존 정책이다.

기존 recovery worker에서 저장한 library_root가 있을 때만 Restore한다. 저장값 부재·읽기 경합에서 사용자 Documents로 fallback하지 않으며 늦은 결과로 입력 수정·CloseWait를 덮어쓰지 않는다. 잘못된 저장 위치는 ready를 막지 않고 폴더 수정이 가능한 오류로 알린다. 기존 Restore의 누락된 저장 디렉터리 생성 정책은 유지한다.

## 경계와 비용

현재 Windows CPU host에 연결한다. 대체 eframe host·Linux/macOS는 수동 입력과 미지원 이유를 제공한다. native UTF16 32767 code unit 경계와 제품 UTF8 32KiB 상한은 별도이며 무손실 문자열로 표시할 수 없는 경로를 적용하지 않는다. Windows picker는 local drive namespace로 한정하고 UNC/device namespace는 오류로 돌려준다.

picker를 열 때만 thread1·COM 객체·proxy1·cap1 channel을 생성한다. 상시 thread나 다운로드 dependency는 추가하지 않는다. 정확한 바이너리 크기·실측 GUI/CPU 결과는 [작업 기록](../work-items/S00-native-folder-picker-layout.md)에 남긴다. 전체 S00 stack·다른 OS·정식 릴리스 인수를 이 결정으로 대체하지 않는다.

## 검증

Windows 전체 검사의 구현·실패 경로 회귀와 private desktop의 선택/취소/원 창 종료/새 process 저장 위치 복원을 요구한다. 실제 PNG는 root가 직접 확인하고 사용자 데스크톱으로 fallback하지 않는다. [IFileOpenDialog 옵션](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-_fileopendialogoptions), [IModalWindow Show](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-imodalwindow-show)를 계약 근거로 사용한다.

최종 scoped 인수: [whole878/0/13](../validation/evidence/windows-msvc-folder-ui-close-safe-workspace-retry-20261007/root-review.json), [native6사례12process](../validation/evidence/windows-msvc-folder-ui-close-safe-workspace-retry-20261007/native-five-root-review.json). 실제 private dialog 선택/취소/ownerclose·새 process 복원과 같은 frame Close/queued completion 회귀를 확인했다. CPU9,081,344B로1차 UI보다40,448B(39.5KiB) 증가했고 dependency/Cargo.lock 변경은 없다. 물리IME/reader·OS DPI·forced OS cleanup 모든 실패의 인수를 뜻하지 않는다.

[추가 실제 위치 실패 보존](../validation/evidence/windows-msvc-folder-ui-close-safe-workspace-retry-20261007/native-invalid-root-review.json)까지 포함하면6사례12process다. 원 선택 후 소유 ASCII 파일을 잘못된 위치로 입력해 기존 행·설정·새 process 복원을 확인했다. Emoji는 native 선택/복원으로 검증하며 합성 키 메시지 입력 실패는 별도 이력이다.
