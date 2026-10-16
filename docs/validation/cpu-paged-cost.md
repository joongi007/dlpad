# CPU paged preview baseline 비용

[실측](cpu-paged-cost-prefix/result.json) exit0. Linux release CPU binary SHA256 `166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417`, 정확한 크기12402040bytes다. [종료 후 검토](cpu-paged-cost-prefix/review.json)에서도 SHA/크기가 일치했다.

실행: `python3 scripts/accessible_performance_probe.py target/release/dlpad-cpu docs/validation/cpu-paged-cost-prefix --font /mnt/c/Windows/Fonts/malgun.ttf`. 기존 probe와 source/Cargo는 수정하지 않았다. private Xvfb/DBus/temp folders 및 기존 폰트 읽기만 사용했다.

WSL2 x86_64 kernel6.18.40.1/AMD Ryzen7 9700X 환경에서 앱 perPID10.013초 idle 표본의 CPU는 단일코어기준0.4993%, 최대RSS54080KiB(52.8125MiB)다. 제안 idle1% 미만/대기80MiB에는 이 표본이 들어간다. root의 두 headless8h soak launch와 별도 공식 도구 설치 GUI가 병행될 수 있는 host load 조건이며 호스트 전체 CPU를 앱 비용으로 합산하지 않았다.

앱은 기본 다운로드 화면이며 파일목록/이력/도구/최대보고서/preview 대형페이지를 열거나 활성 다운로드를 하지 않았다. 단일 표본은 전체기능 peak/8h 누수/다른OS/IME/reader/시작1초/숨김40MiB 인수가 아니다. 기존 단일 표본과의 차이를 확정적인 성능 회귀·개선으로 판정하지 않는다.

owned GUI와 private bus는 회수됐고 임시fixture 삭제도 확인됐다. GUI SIGTERM exit-15는 probe의 소유 프로세스 정리이며 정상 사용자 CloseWait 동작 증거가 아니다. 최종 UI 스택채택/P0 완료를 이 수치로 주장하지 않는다.


## 독립 문서 갱신 검토

원시 result.json의 sample_seconds10.013098584/rss_max_kib54080/idle_cpu_percent0.499345928을 직접 다시 확인했다. 최신 Linux1669 executable은12402040bytes이며 앞선522e parity executable12327288bytes보다74752bytes 크다. 이 차이는 여러 통합 변경을 함께 포함하므로 페이지화 한 기능만의 비용으로 귀속하지 않는다. [Windows native CLI](windows-cpu-paged-preview.md)의 실제7244288bytes executable은 플랫폼별 별도 uncompressed 산출물이며 Linux RSS와 비교 지표로 섞지 않는다. Linux [대형 유효보고서](cpu-report-stress-validation.md)의 HWM70.8MiB는 다른 workload 결과다.
