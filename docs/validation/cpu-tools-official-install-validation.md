# CPU 공식 고정 관리 도구 설치·중단 GUI

2026-10-03 Linux/WSL의 CPU SHA `166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417`, harness SHA `6bbb82efce81a5f4ada58f8d7dbbe7dde238dd8d72d0757bbb298ca7cd3ba469`. 실제 공개 개발 도구 다운로드는 기존 사용자 승인 범위에서 수행했다. 각 case는 새 임시 appdata/media/XDG 폴더, 합성 기존 관리 v1/v2와 외부 sentinel, private Xvfb/DBus를 사용했다. 사용자 desktop/clipboard/실제 미디어/계정/사이트 추출기를 사용하지 않았다.

## 실제 공식 설치

`install-1` exit0/PASS. UI의 고정 버전 설치 action으로 기존 managed::candidate Linux pin을 그대로 설치했다. source는 GitHub yt-dlp release2026.08.19/yt-dlp_linux, exact40,446,224B와 SHA256 `58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a` 일치. 설치 manifest source/version/target/hash와 active를 확인했다. 다운로드된 파일의 explicit `--ignore-config --no-plugin-dirs --version` 실행은 exit0/2026.08.19였다. 영상/쿠키/로그인은 실행하지 않았다. 실제 bytes의 digest는 stream 방식으로 검사하며 바이너리 전체를 메모리에 적재하지 않는다.

명시 Use action 이후 다운로드의 기존 yt-dlp field Text.GetText가 공식 managed path와 일치했다. active 공식버전의 Remove는 native AT-SPI Enabled/Sensitive가 모두 false였다. 합성 v1으로 명시 활성 전환 뒤 inactive 공식버전 Remove를 실행해 공식 installation directory가 사라지고 v1/v2 원 bytes 및 외부 sentinel이 보존됐다. 같은 앱의 선택/제거·종료까지 실제 GUI로 확인했다. 새로운 official install은 임시 소유 관리cache이며 실제 외부 설치를 삭제하지 않았다.

## 실제 네트워크 전송 중단

`cancel-1` exit0/PASS. 별도 새 임시 data에서 공식 설치를 시작했다. `.staging-257416-0/yt-dlp`에16,384B가 실제 기록된 시점(0<bytes<40,446,224)을 관측한 후 cached native Cancel button의 Enabled/Sensitive true 및 visible extents를 확인하고 Action.DoAction을 전달했다. 설치 시작 기준 요청0.054초/ack0.081초 표본이었다. 설치 시작 전 Cancel은 두 state가 false였다.

최종 executable을 publish하지 않았고 소유 staging을 회수했으며 기존 active synthetic-v1, 두 보관 executable bytes, 외부 sentinel은 보존됐다. 실제 cancelled PNG는 설치 중단/이전 active 보존 메시지를 표시했다. pre-cancel unit을 실제 네트워크 취소로 바꾸어 보고하지 않았다. 이번 표본이 모든 HTTP blocking phase의 즉시 취소를 보장하지 않는다. 기존120초 phase deadline과 read 사이 cancellation 정책은 유지한다.

HttpsTransport는 proxy(None)을 명시하므로 환경 CONNECT proxy는 효과가 없다. 이 시험은 별도 proxy/TLS 변경/hosts·보안·전역 설정 변경 없이 직접 공식 HTTPS 전송을 관측했다. child 환경의 기존 proxy 변수는 상속하지 않았다.

## 원시 근거와 검증

[evidence/cpu-tools-official/](evidence/cpu-tools-official/)의 install-1/cancel-1 result.json, activation.sqlite, PNG와 bounded app/bus 로그를 보존했다. 두 case 정상 exit0, binary SHA before/after 동일, 소유 app/DBus 채널 회수. 보존 SQLite integrity_check=ok, active는 정확 synthetic-v1/target이었다. host /proc의 기록 PID/start identity를 직접 비교해 원 app/bus 부재를 확인했다. independent-verification.json에 raw invariant/DB/identity 검증 결과를 기록했다. version-only subprocess도 wait/returncode로 완료를 확인했다. 그 이상의 미관측 descendant별 PID 증거를 주장하지 않는다.

실행 명령: `python3 scripts/cpu_tools_install_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-tools-official/install-1 --expected-sha 166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417 --case install` 및 같은 명령의 output cancel-1/case cancel (각 exit0). `python3 -m py_compile scripts/cpu_tools_install_probe.py`와 문서 검사도 실행했다. 이번 작업은 전용 harness/docs만 변경했으며 제품/Cargo/shared helper/고정pin은 변경하지 않았다.

## 미검증

Windows/macOS 공식 GUI 설치·취소, FFmpeg/JS runtime archive 공급, supply license 패키징, 설치 publication crash, 다른 endpoint failure/지연 phase, 실제 site/미디어 시험과 전체 S04/P0는 미완료다. full official binary는 임시 설치에서 선택적 제거까지 확인하여 보존하지 않았으며 digest/size/version 결과와 raw metadata를 근거로 남겼다. 다른OS/배포를 Linux 한 건으로 대체하지 않는다.
