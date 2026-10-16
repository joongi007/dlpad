# S00 private bus 접근성·실제 Orca 제한 검증

관측: 2026-10-02, Linux X11/Xvfb. 제품 소스·Cargo·시스템 설치·사용자 bus/desktop/audio 설정을 변경하지 않았다.

## 판정

**제품 Iced 접근성 게이트 FAIL.** 소유 private D-Bus/AT-SPI registry에서 제품 앱·한글 입력 필드 트리가 없다. 같은 조건의 egui 대안은 트리를 공개하고 실제 Orca 42.0의 무음 이벤트 처리까지 확인했다. 대안을 최종 제품 스택으로 채택한 결과가 아니며 음성 출력·전체 reader 사용성·Windows/macOS는 미검증이다.

| 실행 | 바이너리 SHA256 / bytes | 실제 관측 |
| --- | --- | --- |
| 최초 Iced tree 비교 | 7e5fe81444a31806189e3a84d51d1ccf809f48bd3c83df1396c9d2fbabacd469 / 9873856 | registry에 제품 앱 없음 |
| 후속 Iced Orca 비교 | 80c3338775a4c0bcc017fb4ad1ef766012667fc4907f98841095b5576b23efae / 9892512 | 제품 앱 없음; Orca runner `-c`만 등록 |
| egui 세 비교 | e99af18a079211c2267336ca9dd228da7847dd9bfd5be7ec583a831e49e4aeaa / 8806208 | 한글 레이블, entry, 텍스트·포커스 공개 |

## 실제 시험과 증거

`python3 scripts/reader_bus_probe.py target/release/dlpad target/release/dlpad-egui-spike docs/validation/reader/orca-xkb-comparison --font /mnt/c/Windows/Fonts/malgun.ttf` 실행은 수집 종료 0이다. 이 값은 제품 게이트 통과를 의미하지 않는다.

- [최초 reader 없는 비교](reader/tree-comparison/comparison.json): Iced tree 없음. egui 실제 GetText는 `readerfixture`였고 하이픈 합성 키가 누락되어 원 `fixture_text_accessible=false`가 남았다. 새 시험은 알파벳만 사용했다.
- [Orca 첫 실패](reader/orca-comparison/comparison.json): 실제 orca.orca.main 진입 후 xkbcomp 부재로 실패했다. stderr/원 로그를 보존했다.
- [의존성 보완 후 원 비교](reader/orca-xkb-comparison/comparison.json), [수집 지표의 범위 정정](reader/orca-xkb-comparison/review.json).
- [egui 실제 Orca 로그](reader/orca-xkb-comparison/egui/reader.log): `ATSPI registry initialized`, 한글 입력 검증 entry의 focus·text-changed·caret, `readerfixture`, Tab 이후 다운로드 push button의 focused 이벤트 처리. reader 정상 종료 0, 로그 109799B.
- [Iced 실제 Orca 로그](reader/orca-xkb-comparison/iced/reader.log): registry 초기화 완료 없음. 종료 요청 deadline 이후 소유 reader SIGKILL 회수(-9), 로그 813B. tree의 `-c` 노드는 reader 자체이며 제품 공개로 세지 않는다. startup의 alive만으로 초기화 성공을 판단하지 않는다.
- 각 후보 폴더의 input.png/tab.png와 result.json에 텍스트·역할·GetState 원 관측을 보존했다. 모든 소유 직접 자식 회수/임시 fixture 삭제 기록이 있다.

원 JSON의 `tree_published`는 전체 registry node 유무였고 `screen_reader_verified`는 무음 이벤트 처리만 뜻했던 수집기 오류가 있었다. 원 데이터를 수정하지 않고 review.json으로 구분했다. 최종 스크립트는 제품 이름을 확인하고 `reader_event_processing_verified`로 한정하며 전체 `screen_reader_verified`는 false로 유지한다. 따라서 egui의 실제 reader 이벤트 처리는 확인되었지만 전체 reader 인수 조건 통과를 주장하지 않는다.

## 격리·상한

신규 소유 Xvfb, 별도 dbus-daemon/AT-SPI bus, temp XDG data/config/cache/runtime와 app-data/media sibling, GSETTINGS_BACKEND=memory만 사용했다. 기존 폰트는 읽기만 한다. visible desktop fallback이 없다. 공식 Orca CLI는 다른 Orca를 shell/pgrep으로 조사하는 경로가 있어 실행하지 않고 공식 Python 모듈의 main을 전용 runner에서 호출했다. private prefs의 enableSpeech/Braille/Sound/MouseReview=false로 backend 초기화를 끈 실제 무음 Orca 시험이다. 다른 사용자 reader를 검색·종료하지 않았다.

reader 로그/표준 출력 파일은 RLIMIT_FSIZE 2MiB, 요약 이벤트는 최신 64개·각 500자. 재사용 tree 수집은 최대 48노드·호출별 timeout 5초·출력 상한이 있다. 직접 소유 process group만 종료한다. 이 시험의 main 전체 wall-clock에는 별도 강제 deadline이 없으며 호출 상한 합으로 제한된다. 사용자 bus/음성 출력은 사용하지 않았다.

## 프로젝트 전용 공급과 비용

[정확한 12개 패키지 pin·공식 Ubuntu URL·SHA256](reader/packages.json)에 다운로드 파일 해시와 apt-cache 공식 metadata SHA256 일치를 기록했다. `apt-get download` 및 `dpkg-deb -x`만 사용하여 .local-tools/reader에 추출했고 dpkg 시스템 설치는 하지 않았다. 저장소 metadata는 환경에 이미 있는 apt index이며 이번 작업에서 freshness/signature를 별도로 갱신 검증하지 않았다. 다운로드 합계 1234498B, 추출 regular file 합계 7319231B다. 각 패키지의 usr/share/doc copyright를 함께 보존했다. Orca는 LGPL-2.1+이며 GIR/pyatspi/x11 도구는 해당 copyright 파일을 따른다. 개발 시험 도구 비용이며 제품 의존성/배포물에는 추가하지 않았다.

추출된 런타임 모듈은 기존 시스템 GI/GTK/AT-SPI 라이브러리와 함께 사용한다. speech-dispatcher/espeak 등 음성 backend는 설치하지 않았다.

## 다음 게이트

현재 Iced renderer 경로의 native 접근성 export를 구현하거나 실제 접근성 tree·reader 이벤트를 제공하는 대안을 제품 수준에서 검증해야 한다. 후자는 현재 spike의 증거에 한정되므로 제품 기능 통합/3 OS reader/실제 음성 또는 승인된 dummy speech 출력/키보드 탐색 전체 게이트가 남는다. Iced reader 초기화 미완료의 원인을 tree 부재로 단정하지 않는다. 제품 소스 수정 및 스택 결정은 통합 담당 범위다.
