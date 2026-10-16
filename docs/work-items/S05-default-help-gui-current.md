# S05: 기본 native UI 도움말 격리 화면 검증

- 상태: completed (기본 도움말 화면 가시성 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: root; 이 문서 및 p0-default-help-gui-first evidence
- 선행 작업: 기본 도움말 구현 및 현재 기본release 성공

## 사용자 결과

기본 숫자 품질·구간·자르기·FFmpeg 안내를 실제 native 화면에서 확인한다.

## 계약과 범위

현재release immutable 사본/한글폰트/privateXvfb/합성fixture만 사용한다. 기본UI와CPU후보를구분한다. 다른OS/스크린리더/IME전체인수로확대하지 않는다.

## 인수 조건

source 바이너리 SHA보존/앱정상종료/소유fixture회수 및 실제스크린샷 가시성 확인. 동시build 중 짧은CPU/RSS 표본은 전체성능인수로표시하지 않는다.

## 검증과 handoff

실행 중.

첫 격리 화면에서FFmpeg요약/기본/예/충돌/미확인버전을root직접읽었다. 기본화면높이가늘어품질/구간/자르기는스크롤인수가필요하다. root담당scripts/native_probe.py와x11_diagnostics.py에0..30한정--scroll-steps를추가하고desktop/scroll-end동시입력을거절한다. 기존scroll-to-end기본30은유지한다. 새사본을만들지않고동일immutable기본바이너리의중간화면을새evidence에서시험한다.

Root가첫FFmpeg,scroll3숫자품질,scroll6구간/자르기화면을직접읽었다. 실제각실행exit0/3초표본/privateXvfb/자기childreaptrue이며품질전용0/144~4320·예1080·오디오미지원/변환없음과기본/충돌/미확인버전의정합성을확인했다. 기본바이너리10082464bytes immutable사본을사용했다. 위CPU도움말5개와기본Iced화면은다른binary/fixture다. keyboard/reader/IME/System이벤트/다른OS/전체성능인수는남는다.
