# CPU 테마·배율 네이티브 검증

2026-10-03 Linux CPU SHA `608eeeb5a2f3a046e0a389ca788770f58e6ff38369d056ef8c8debdb262829c9`, 12,403,064B. 고정 snapshot의 960×640 창에서 실행했다. root의 후속 guardian/layout source를 이 결과로 검증 완료라고 표현하지 않는다.

## 608eeeb 수정 전 결과

| 테마 | 150% | 200% |
| --- | --- | --- |
| 밝게 | 입력·취소·Tab 제한 scope PASS | 필수 설정·IME 잘림 FAIL |
| 어둡게 | 입력·취소·Tab 제한 scope PASS | 필수 설정·IME 잘림 FAIL |
| 시스템 | 입력·취소·Tab 제한 scope PASS | 필수 설정·IME 잘림 FAIL |

실제 ComboBox 중심을 소유 화면에서 클릭하고 보이는 선택 항목 Action으로 테마와 배율을 선택했다. theme.sqlite readonly 조회에서 각각 JSON `"Light"`, `"Dark"`, `"System"` 값을 확인했다. 초기 System을 재선택한 것만으로 저장이라고 주장하지 않도록 먼저 반대 테마를 선택했다. 재시작 후 테마 복원은 이 harness의 검증 범위 밖이다. 배율은 기존 계약대로 세션 한정이며 DB 저장을 요구하거나 구현하지 않았다.

150% 실제 role79 IME 입력란은 (12,495,936,48), 상태 패널 상단 y549보다 위다. 필수 테마·배율·인터페이스·관측 적용도 보이는 영역 안에 있었다. 실제 Text.GetText로 ASCII `abc`, 한글 확정 `한글 `, 조합 취소 뒤 `한글 `, Latin 전환 `한글 abc`, 선택 대체 `xy`를 확인했다. 실제 Tab 뒤 다운로드 role43 버튼의 Focused state를 확인했다. 세 테마의 preedit.png를 직접 검토해 한글 font와 입력란 내부 한[글] 조합 선택 색상을 확인했다. raw XIM Preedit callback/preedit_verified/ime_verified=false는 원 collector 값으로 유지했고 전체 IME 성공과 구분한다.

200% 실제 테마·배율·인터페이스 control은 보이지만 상태 패널 상단 y519 아래로 관측 적용(16,551,129,64)과 IME(16,662,928,64)가 잘린다. 실제 입력란은 창640px 밖이다. scaled.png에서도 상태 패널이 설정 도움말 중간부터 덮는 것을 확인했다. 창 확대·숨겨진 Action·하드코딩 좌표 입력으로 통과시키지 않고 오류/exit1을 보존했다. 따라서 200% 실제 한글 입력과 Tab은 미검증이며 레이아웃 인수는 실패다. 150%에서도 IME 아래 안내문이 잘린다. root 소유 settings scroll 구현과 새 binary의 동일 크기 재검증이 필요하다.

## 명령·격리·정리

각 조합에 대해 `python3 scripts/cpu_scaled_native_probe.py target/release/dlpad-cpu docs/validation/evidence/cpu-scaled-native/<theme>-<scale>-1 --theme <System|Light|Dark> --scale <150|200> --expected-sha 608eeeb5a2f3a046e0a389ca788770f58e6ff38369d056ef8c8debdb262829c9 --font /mnt/c/Windows/Fonts/malgun.ttf`를 실행했다. 150%는 세 case exit0, 200%는 세 case exit1이다.

[evidence/cpu-scaled-native/](evidence/cpu-scaled-native/)에 원 result.json/PNG/log와 별도 matrix/independent-verification을 보존했다. private Xvfb/DBus/Fcitx/proot와 고유 임시 XDG/appdata/media만 사용했다. 사용자 desktop/clipboard/실제 media/계정/음성은 사용하지 않았다. 직접 자식 stop/wait, 임시 폴더 회수와 binary SHA 전후 동일을 확인했다. 원 collector에 개별 PID identity는 없으므로 PID/start 독립 증거라고 쓰지 않는다. 별도 검증은 고유 runtime prefix에 대응하는 /proc 명령행 잔존 여부를 조회한다. 다른 장기 시험 PID에는 개입하지 않았다.

전용 harness는 기존 accessible_ime_probe의 격리 setup과 입력 절차를 전용 파일로 재사용하고 공유 harness를 변경하지 않았다. 제품·Cargo·README/P0status 변경은 없다. Windows/macOS, 전체 reader, 제품 재시작 테마 복원, 배율200% 입력은 미검증이다. P0/S00 전체 완료를 뜻하지 않는다.

후속 재검증용 collector에 소유 자식 PID/start identity와 로그 파일 2MiB 상한을 추가하고 구문 검사를 통과했다. 기존 6case 실행에는 이 추가가 없었으므로 이전 결과에 새 증거를 소급하지 않는다.

## 설정 스크롤 수정 후 1fe649 검증

root가 settings-content ScrollArea를 추가한 immutable CPU `/tmp/dlpad-settings-scroll-binary-cgg8vee9/dlpad-cpu` SHA `1fe649add2558aca14b30785b7498fa4820986099e639d15ea5f716e86689f0e` /12,404,344B를 읽기만 사용했다. root의 target/release 재빌드와 이 실행을 분리한다. 실행 binary와 전용 harness SHA는 6case 모두 같았다. 현재 source 일부의 실행 중/후 관측은 별도 scroll-source-observation-before/after.json에 저장했고 관측한5파일은 같았다. 해당 관측을 전체 source freeze나 정확한 컴파일 입력 증거로 확대하지 않는다.

| 테마 | 150% | 200% |
| --- | --- | --- |
| 밝게 | 실제 휠·입력·취소·Tab PASS | 실제 휠·입력·취소·Tab PASS |
| 어둡게 | 실제 휠·입력·취소·Tab PASS | 실제 휠·입력·취소·Tab PASS |
| 시스템 | 실제 휠·입력·취소·Tab PASS | 실제 휠·입력·취소·Tab PASS |

6case 모두 창은960×640였다. 초기 실제 theme/scale dropdown 선택과 테마 DB 값 확인 후 private Xvfb에 실제 휠3회를 전달했다. 150% IME는 y495→456/h48, 안내문 y509/h26, status y549였다. 200% IME는 y662→394/h64, 관측 적용 y551→283/h64, 안내문 y732→464/h35, status y519였다. lower control의 전체 bounds와 PNG로 안내문/관측 적용/IME가 콘텐츠 영역 안에 들어온 것을 확인했다. 숨겨진 Action이나 창 크기 변경은 사용하지 않았다.

6case에서 ASCII/한글 확정/조합 취소/Latin 전환/선택 교체는 위와 동일한 실제 Text.GetText를 통과했다. 각 preedit.png 직접 검토에서 한[글] 조합 선택과 읽을 수 있는 한글을 확인했다. Tab 뒤 Focused 대상은 새로운 ScrollArea의 role67 scrollpane였다. 이전 다운로드 focus 결과와 구분하여 그대로 기록했다. 휠 접근과 실제 Tabfocus를 검증했으며 PageUp/PageDown 등 키보드 스크롤 전체 흐름·전체 reader·원시 XIM callback·음성은 미검증이다.

명령은 이전과 같은 인자에 binary 경로를 위 immutablecopy, expected-sha를1fe649..., output을 `scroll-<theme>-<scale>-2`로 지정했다. 6개 모두 exit0/제한 Linux 범위 PASS. scroll-matrix.json 및 각 independent-verification.json에 host /proc로 app/IME/bus의 PID/start 원본 부재와 고유 runtime prefix 잔존0을 독립 확인했다. private display/임시 fixture는 회수했고 immutablecopy 정리는 소유자 root에게 맡긴다.

처음 `scroll-*-1` 6개는 collector의 app전체 RLIMIT_FSIZE 적용 상태에서 GUI가 초기 종료한 실패 raw다. 앱 데이터 파일까지 제한하는 방식 대신 app stdout만64KiB 보존/나머지 drain하도록 바꾸자 같은 immutable binary에서 `-2` 6개가 통과했다. 이 collector 실험 실패를 제품 회귀와 혼합하거나 삭제하지 않는다. bus/IME 로그는2MiB 파일 상한이며 app 데이터에는 이 파일크기 제한을 적용하지 않는다. 기존608eeeb 제품 잘림 raw 역시 불변으로 유지했다.

Python 구문 검사와 문서 검사322개를 통과했다. Windows/macOS, 음성/전체 reader, 재시작 테마 복원과 이후 서비스 backend source는 이 검증으로 합격 처리하지 않는다.
