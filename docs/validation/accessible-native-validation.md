# S00: 실제 접근성 후보 IME·무음 Orca 검증

관측: 2026-10-02, Linux X11/Xvfb. dlpad-accessible 후보의 실제 엔진 연결 바이너리에 한정한다. 기존 egui spike 또는 Iced 결과를 후보의 성공으로 대체하지 않는다. 전체 P0·3 OS·전체 스크린리더 사용성 합격을 뜻하지 않는다.

## 현재 실제 관측

첫 후보 SHA256 `0006783276df0485f25bfeec0af805389cb81ac0e35d9b25116243cf9c823dd1`, 10416792B. 다운로드 일시 중단 연결 오류 수정 전 바이너리다. 수정 후 핵심 재검증은 아래 최종 증거로 분리했다.

[실제 레이아웃 inspect](accessible-native/inspect/result.json)의 설정 입력 필드는 y=102였다. [첫 reader 좌표 실패](accessible-native/reader/comparison.json)는 가설 y=125가 입력 필드 밖에 있어 텍스트 입력을 못했다. 그대로 보존하고 inspect 확인 후 y=102로 다시 시험했다.

[IME 원 관측](accessible-native/ime-full/result.json)과 [시각 검토](accessible-native/ime-full/review.json): ASCII abc, 한글 Space 확정 `한글 `, BackSpace 두 번의 조합 취소 후 `한글 `, Latin 전환 `한글 abc`, 선택 교체 `xy`를 AT-SPI Text.GetText로 확인했다. [preedit.png](accessible-native/ime-full/preedit.png)를 직접 보니 입력 칸 안에 한글이 표시되고 마지막 글에는 조합 선택 표시와 caret이 있다. preedit GetText도 `한글`이다. 제품 raw IME 이벤트 trace 기능은 없어 raw Preedit 이벤트가 도착했다고 주장하지 않는다. Clipboard를 사용하지 않았다. 수집기는 시각 판정을 자동으로 하지 않으므로 원 preedit_verified/ime_verified=false와 exit1을 유지하고 실제 이미지 판정을 review.json에 분리했다.

[무음 reader 원 관측](accessible-native/reader-corrected/comparison.json), [제한 판정](accessible-native/reader-corrected/review.json), [실제 Orca 로그](accessible-native/reader-corrected/candidate/reader.log): 한글 레이블·entry role·GetText `readerfixture` 공개, Orca registry 초기화, entry의 text-changed/caret 처리, 실제 Tab 후 entry→다운로드 push button의 locusOfFocus 이동을 확인했다. Orca exit0, 로그 89216B. 후보에는 mirror label이 없어 Orca 로그에 전체 readerfixture 문자열이 등장하지 않는다. 원 문자열 검색 false를 보존하고 text-changed 실제 처리·AT-SPI 텍스트·키보드 focus 처리로 한정했다. 무음 이벤트 처리 성공이며 실제 음성이나 전체 reader 사용성 성공은 아니다.

## 수정 후 최종 바이너리 재검증

최종 SHA256 `55ecdbc89c4eb42c4e898e939e7ca164e94ed5359848a903c7d58d77a452401f`, 10416792B. root의 Pause 연결 수정 후, 다른 source/build 변경 없이 두 독립 소유 화면·fixture에서 IME와 reader를 수집했다. [전체 GUI 종료 후 hash 확인](accessible-native/binary-stability.json)도 동일했다.

[최종 IME 원 관측](accessible-native/ime-final/result.json)과 [별도 시각 판정](accessible-native/ime-final/review.json): commit/cancel/Latin/selection 모두 첫 후보와 동일한 기대값이다. [최종 preedit.png](accessible-native/ime-final/preedit.png)를 직접 확인하여 입력 칸 내부의 한글·마지막 글의 조합 선택 표시·caret을 다시 확인했다. 원 collector exit1/시각 미판정 false를 보존했다. 이는 시각 검토를 자동수집 결과와 구분하기 위한 값이다.

[최종 reader 원 관측](accessible-native/reader-final/comparison.json)은 entry→중간 frame→다운로드 버튼의 실제 focus 이동을 직접 entry→button 문자열 조건 때문에 false로 수집했다. [원 로그의 단계별 검토](accessible-native/reader-final/review.json)에 두 locusOfFocus 로그 위치를 남겼다. predicate는 중간 frame 유무와 무관하게 최종 다운로드 버튼의 실제 locusOfFocus 처리로 보수적으로 수정했다.

[최종 reader 재확인](accessible-native/reader-final-confirm/comparison.json), [최종 실제 reader 로그](accessible-native/reader-final-confirm/candidate/reader.log): candidate tree·한글 label·GetText readerfixture, registry 초기화, text-changed 실제 처리, keyboard focus 실제 처리 모두 true. collector exit0, reader exit0, 로그 83188B. [input.png](accessible-native/reader-final-confirm/candidate/input.png)를 직접 확인하여 실제 입력 문자열이 화면과 AT-SPI에 일치하는 것도 대조했다. 전체 screen_reader_verified와 actual_audio_used는 false로 유지했다.

최종 실행은 위 명령의 출력 폴더를 ime-final/reader-final/reader-final-confirm으로 바꾸고 기본 input point 300,102를 사용했다. scripts py_compile 및 --validate-only 통과, check_docs 통과. 제품/Cargo를 변경하거나 바이너리를 빌드하지 않았다.

## 실행과 격리

`python3 scripts/accessible_ime_probe.py target/release/dlpad-accessible docs/validation/accessible-native/ime-full --font /mnt/c/Windows/Fonts/malgun.ttf --input-point 300,102`

`python3 scripts/accessible_reader_probe.py target/release/dlpad-accessible docs/validation/accessible-native/reader-corrected --font /mnt/c/Windows/Fonts/malgun.ttf`

모든 앱은 소유 Xvfb/private D-Bus/AT-SPI/temp XDG/GSettings memory와 새 app-data/media sibling에서 실행했다. 기존 폰트는 읽기만 한다. 사용자 desktop·clipboard·audio 접근이 없고 visible fallback도 없다. 기존 프로젝트 전용 Fcitx와 공식 Ubuntu Orca 모듈을 재사용했다. 제품/Cargo/build 및 다른 담당 다운로드 probe를 변경하지 않았다.

Orca는 공식 CLI의 기존 다른 reader 검색/종료 경로를 피하고 공식 모듈 main을 private prefs enableSpeech/Braille/Sound/MouseReview=false로 실행한다. 파일 로그는 2MiB OS 상한, 요약 64개×500자, tree 48노드 및 gdbus timeout/출력 상한을 유지했다. 새 reader 로그는 line buffering을 사용한다. 이전 Iced reader 로그의 초기화 완료 미관측에는 buffering 영향도 있을 수 있어 그 원인을 tree 부재로 단정하지 않는다. 이 후보 로그는 실제 처리/정상 종료를 확인한다. 모든 소유 직접 자식을 회수하고 임시 fixture를 삭제했다.

## 남은 게이트

일시 중단 수정 후 바이너리의 핵심 IME·무음 reader 재검증을 완료했다. 제품 다운로드 화면 전체 keyboard 순서·실패 보고서의 reader 사용자 경험·음성 출력·Windows/macOS·background/tool/library parity는 이 시험에서 완료하지 않았다. 스택 확정과 공통 문서 갱신은 통합 담당 범위다.
