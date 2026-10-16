# S05: 현재 옵션 GUI 시험의 명시 바이너리 pin

- 상태: completed (격리 pointer/AT-SPI 펼침 및 전체 문단 가시성 범위)
- 분류: validation-support
- 우선순위: P0
- 요구사항: R07, R10
- 담당/수정 범위: root; scripts/cpu_options_probe.py 및 이 문서
- 선행 작업: S05-p0-option-help-completion

## 사용자 결과

새 옵션 도움말 snapshot을 기존 합성 GUI 시험으로 검증한다.

## 계약과 범위

옛 고정 SHA 기본값은 유지하고 명시 --expected-sha256을 정확64hex로 검증해 새로운 immutable 바이너리를 시험한다. 변경된 시작 label을 맞춘다. Xvfb private 시험만 수행하며 visible fallback, Documents, 실제영상은 사용하지 않는다.

## 인수 조건

잘못된 SHA를 입력 단계에서 거절하고 매 case 전후 실제 SHA 일치를 확인한다. 새 바이너리의 옵션·충돌·DB 보존 검증은 실제 실행으로 별도 기록한다.

## 검증과 handoff

준비 중. Python 문법 검증과 실제 GUI 성공을 구분한다.

Root 실제 Python 문법 검증 exit0 및 subprocess malformed SHA 입력 exit2/output 미생성을 확인했다. 아직 GUI 인수는 실행하지 않았으며 새 CPU 바이너리 빌드/명시 SHA pin 뒤 별도 수행한다.

추가 help case는 private Xvfb에서 다섯 도움말을 실제 클릭하고 AT-SPI expandable/expanded 상태 및 스크린샷/트리를 보존한다. 닫은 뒤 다음 도움말을 확인하며 Job/worker 미생성을 검증한다. 내용 읽기·가시성은 root가 실제 결과를 검토하며 단위시험으로 대신하지 않는다.

첫 실제 help GUI는 FFmpeg/cookie 펼침 두 개를 확인한 뒤 품질 도움말의 게시 좌표 y564가 실제 footer(y488 이후)에 가려져 클릭이 실패했다. failure.png/native bounds를 root가 대조했다. product를 변경하지 않고 lookup이 실제 다운로드 시작 버튼 bounds에서 content 하단을 계산하도록 수정한다. 첫 실패/cleanup/hash일치는 보존한다.

Second help GUI는5개 도움말의 실제 AT-SPI expandable/expanded 상태와 native 클릭/스크롤을 통과했고 source 바이너리 SHA ca8e969f... 전후일치/ownedcleanup true/Job·worker미생성을 확인했다. root는 품질·자르기 스크린샷에서 예시·기본값·충돌·미확인 버전이 footer 위에 표시되는 것을 직접 확인했다. 다른 도움말 내용과 기본Iced UI·keyboard 전체 인수는 별도로 확인한다.

Root가 나머지 화면도 직접 읽었으며 구간 도움말의 마지막 충돌/버전 문단이 second screenshot에서 footer 아래로 잘린 것을 확인했다. 펼침 시험은 통과했으나 전체 가시성 인수로 확대하지 않는다. probe가 실제 마지막 문단 bounds 전체를 footer 위로 스크롤해 screenshot을 보존하도록 보강하고 third에서 인수한다.

Third help5개 실제PASS/ownedcleanup true/원byte SHA전후일치/Job·worker미생성,모든마지막문단bounds전체footer위가시성을root JSON·스크린샷으로확인했다. genericlookup은실제footer위전체rect를요구한다. 키보드전체/실제screenreader/다른OS는별도인수다.
