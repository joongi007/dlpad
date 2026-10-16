# S00: CPU 후보 실제 IME·무음 Orca 제한 검증

관측: 2026-10-03, Linux X11/Xvfb. 후보 `dlpad-cpu` SHA256 `28b412773a5fc1434f86d396667a30e7b6d671a8e30affbe1a41cb1ecea7d658`, 10171256B. [전체 GUI 종료 후 동일 hash 확인](cpu-native/binary-stability.json). 제품 소스·Cargo·바이너리 빌드를 변경하지 않았다.

## 실제 결과

[inspect](cpu-native/inspect/result.json)와 initial.png를 먼저 직접 확인하여 설정 입력 필드 y=102를 확인했다. 이후 IME와 reader는 별도의 소유 temp/Xvfb/private bus에서 독립 수집했다.

[IME 원 관측](cpu-native/ime/result.json), [별도 시각 검토](cpu-native/ime/review.json): ASCII `abc`, Space 확정 `한글 `, 조합 중 BackSpace 두 번 취소 후 `한글 `, Latin 전환 `한글 abc`, 전체 선택 교체 `xy`를 AT-SPI Text.GetText로 확인했다. [preedit.png](cpu-native/ime/preedit.png)를 직접 확인하여 입력 칸 안의 한글 및 마지막 글의 조합 선택 표시·caret을 확인했다. preedit GetText도 `한글`이다. Clipboard를 사용하지 않았다.

원 수집기는 시각 검토를 자동으로 하지 않으므로 preedit_verified/ime_verified=false와 exit1을 그대로 보존했다. 이는 수집 중단이나 commit 실패를 의미하지 않으며 별도 review.json에 실제 이미지 판정을 기록했다. CPU 후보에 raw IME event trace UI가 없으므로 raw Preedit 이벤트를 검증했다고 주장하지 않는다.

[실제 reader 원 결과](cpu-native/reader/comparison.json), [전체 노드·상태](cpu-native/reader/candidate/result.json), [실제 Orca 로그](cpu-native/reader/candidate/reader.log): dlpad-cpu 앱의 한글 label/entry role·GetText readerfixture, Orca registry 초기화, text-changed·caret 실제 이벤트 처리, Tab 후 한글 entry→다운로드 push button locusOfFocus 이동을 확인했다. reader_event_processing_verified=true, 전체 screen_reader_verified=false. 수집 exit0/Orca exit0, 로그 87174B. [input.png](cpu-native/reader/candidate/input.png)를 직접 확인하여 실제 화면 문자열과 GetText 일치를 대조했다. 전체 문자열이 mirror label로 복제되지 않으므로 reader_text_observed=false는 로그의 전체 문자열 검색만 뜻하며 이벤트 처리 실패를 뜻하지 않는다.

## 실행·격리

`python3 scripts/accessible_ime_probe.py target/release/dlpad-cpu docs/validation/cpu-native/inspect --font /mnt/c/Windows/Fonts/malgun.ttf --inspect`

`python3 scripts/accessible_ime_probe.py target/release/dlpad-cpu docs/validation/cpu-native/ime --font /mnt/c/Windows/Fonts/malgun.ttf`

`python3 scripts/accessible_reader_probe.py target/release/dlpad-cpu docs/validation/cpu-native/reader --font /mnt/c/Windows/Fonts/malgun.ttf`

기존 전용 probe를 수정 없이 재사용했다. 새 owned Xvfb/private D-Bus/AT-SPI/temp XDG/GSettings memory와 app-data/media sibling만 사용했다. 폰트와 기존 프로젝트 전용 Fcitx/Ubuntu Orca 도구는 읽기만 했다. 실제 미디어·사용자 Documents/desktop/clipboard/audio를 사용하지 않았고 visible fallback도 없다.

공식 Orca 모듈 main을 private prefs의 Speech/Braille/Sound/MouseReview=false로 실행했다. 다른 사용자 reader 검색/종료 경로를 실행하지 않았다. reader 로그는 2MiB 파일 상한·요약 64개×500자, tree 48노드·gdbus timeout/출력 상한을 유지한다. 소유 직접 자식 회수 및 임시 fixture 삭제는 양쪽 원 JSON 모두 true다. 공급 pin·license·개발 도구 비용은 기존 [reader 설치 검증](reader-validation.md)에 있다. 새 의존성 설치는 없다.

## 인수 범위와 남은 게이트

CPU 후보의 이 한글 입력 시나리오 및 무음 실제 reader 이벤트 처리 범위는 확인했다. 전체 reader 사용자 경험·음성 출력·다운로드 화면 전체 keyboard 순서·보고서 탐색·Windows/macOS·전체 제품 parity는 미검증이다. GL 미사용·메모리·성능 실측은 주 담당의 별도 증거 범위이며 이 결과로 추정하지 않는다. 전체 P0 또는 최종 3 OS 스택 채택을 뜻하지 않는다.
