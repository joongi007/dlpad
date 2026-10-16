# CPU 한글 입력·무음 Orca 회귀

2026-10-03 Linux private Xvfb/DBus. 검증 snapshot은 CPU SHA `166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417` /12,402,040B다. 이후 root의 metadata/UI source 변경과 새 release를 이 결과로 검증 완료라고 표현하지 않는다. S00 전체 스택/3OS/P0/음성 reader 합격을 뜻하지 않는다.

## 실제 한글 입력

전용 cpu_native_recovery_probe.py는 기존 accessible_ime_probe.run을 readonly import했다. 프로젝트 전용 Fcitx5-hangul/proot, private XIM on-the-spot 설정과 임시 XDG/appdata/media를 사용했다. 과거 y102 좌표를 재사용하지 않고 native role79 이름 `한글 입력 검증`의 extents(8,311,944,32)를 조회하여 중심(480,327)에 소유 Xvfb key input을 전달했다.

actual ASCII `abc`, IME key sequence gksrmf와 Space 확정 `한글 `, BackSpace 두 번 조합 취소 뒤 `한글 `, Latin 전환 `한글 abc`, 선택 대체 `xy`를 AT-SPI Text.GetText로 확인했다. fixture commit fallback/raw-event opt-in은 모두 false였다. clipboard를 사용하지 않았다.

preedit 시점의 actual role79 Text는 `한글`이다. preedit.png 직접 검토에서 입력 칸 내부 한글과 마지막 `글`의 조합 선택 색상을 확인했다. 원 collector의 preedit_verified/ime_verified/raw_preedit_event_verified=false 및 exit1은 시각 미판정 원시값으로 그대로 보존했다. 별도 ime/review.json에 실제 text와 시각 판정을 기록했다. screenshot만으로 전체 IME 성공을 선언하거나 raw Preedit callback 수신을 추측하지 않았다. 사용자 전체 입력 흐름/모든 배율/다른OS IME는 미검증이다.

## 실제 keyboard focus와 무음 reader

새 reader case는 공식 Orca 모듈 main을 private prefs에서 speech/braille/sound/mouse review=false로 실행한다. 기존 CLI의 다른 reader 검색·종료 경로를 사용하지 않는다. reader 로그는 resource.RLIMIT_FSIZE 2MiB 상한/line buffering으로 기록했다. 실제 reader.log91,008B에 registry 초기화, text-changed:insert 처리와 entry에서 다운로드 push button으로 locusOfFocus 변경이 존재한다.

동일 named role79/extents로 실제 readerfixture key input을 전달했다. Text.GetText=`readerfixture`, focused state=true를 확인했다. 실제 Tab 이후 focus는 role43 `다운로드` 버튼이었고 Orca가 이를 처리했다. reader exit0, reader_event_processing_verified=true. 실제 audio나 전체 screen_reader_verified는 false로 유지한다. AT-SPI tree만 게시된 상태를 reader 사용성 합격으로 대체하지 않았다. input.png/tab.png와 원 로그, reader/review.json을 함께 따른다.

## 실행·격리·종료

`python3 scripts/cpu_native_recovery_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-native-recovery/current-1 --expected-sha 166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417 --case both` → wrapper exit0/제한 scope PASS. 하위 IME raw exit1은 위 별도 시각 미판정 때문에 유지했다. 무음 reader case PASS와 구분한다.

[evidence/cpu-native-recovery/current-1/](evidence/cpu-native-recovery/current-1/)에 raw JSON/PNG/log/review를 보존했다. SHA before/after 동일, 두 private GUI 종료/임시 fixture 회수. reader app/reader/bus의 PID/start identity를 host /proc로 직접 확인하여 원 프로세스 부재를 검증했다. 두 고유 owned runtime prefix를 사용하는 process도 없음을 owned-runtime-cleanup.json에 기록했다. 기존8h PID/사용자 desktop·bus·clipboard·audio를 건드리지 않았다. 원 IME helper는 직접 자식의 stop/wait를 수행하지만 개별 IME PID는 원 collector에 없으므로 reader identity 증거와 섞지 않는다.

제품/shared harness/Cargo/README/P0status는 변경하지 않았다. 전용 harness Python 구문 검사와 문서 검사를 실행했다. 정확한 source/hash 시점은 이 snapshot에만 대응하며 이후 source를 같은 filename이라는 이유로 성공이라고 표시하지 않는다.

## 남은 인수

실제 음성/전체 keyboard 흐름/진단·목록·managedtools reader 사용성/모든 배율/Windows/macOS/최종 renderer 스택 인수와 새 metadata snapshot 회귀는 남는다. 한 번의 Linux 기본배율 회귀가 전체 release gate를 충족하지 않는다.
