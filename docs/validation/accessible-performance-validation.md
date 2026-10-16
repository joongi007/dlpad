# 접근성 native 후보의 대기 비용 실측

2026-10-02. dlpad-accessible SHA256 `0006783276df0485f25bfeec0af805389cb81ac0e35d9b25116243cf9c823dd1`, 10,416,792B. Rust native eframe/Glow candidate, 960×640, Windows Malgun 기존 로컬 폰트, Linux/WSL private Xvfb·private D-Bus·임시 XDG/appdata. 제품 또는 시스템 renderer 설정을 변경하지 않았다. 장비는 AMD Ryzen 7 9700X, Linux 6.18.33.2-microsoft-standard-WSL2 x86_64/glibc2.35, /proc MemTotal31,616,656KiB다. 사용한 Malgun 파일은13,459,196B/SHA256 `7a183cf1c6c56b9609fcc16eda8b5229fbc11758a21e669ec00343239b02192f`; 파일을 배포물에 묶지 않았다. software driver 이름은 owned child에 요청한 환경 설정이며 GL_RENDERER 문자열을 별도로 조회한 결과가 아니다.

| owned child 설정 | 최대 Linux RSS | 단일 코어 idle CPU | 80MiB 제안 budget |
|---|---:|---:|---|
| 기본 | 196,156KiB | 0.0% / 10.127초 | FAIL |
| LIBGL_ALWAYS_SOFTWARE=true, llvmpipe | 138,204KiB | 0.0% / 10.046초 | FAIL |
| LIBGL_ALWAYS_SOFTWARE=true, softpipe | 247,564KiB | 0.0% / 10.109초 | FAIL |

[기본 원시 결과](accessible-performance/current/result.json), [llvmpipe](accessible-performance/software/result.json), [softpipe](accessible-performance/softpipe/result.json)는 같은 binary SHA이며 모든 GUI/bus 종료 회수와 temporary 제거를 확인했다. 강제 SIGTERM은 측정 fixture 정리이며 제품 정상 종료 인수가 아니다. RSS는 shared library와 driver도 포함한다. PSS·private 값을 RSS 예산과 바꿔 달성했다고 주장하지 않는다. 최초 기본 표본에는 smaps가 없고 후속 2개에 매핑 상위20개를 추가했다.

llvmpipe 매핑에서 libLLVM-15 RSS57,580KiB, anonymous37,552KiB, heap13,540KiB, swrast9,784KiB가 관측됐다. softpipe는 heap139,940KiB로 더 높았다. 이 매핑 관측은 렌더러 비용을 분리할 근거이며 heap 내 모든 allocation의 원인까지 증명하지 않는다. 후속 Rust native renderer 선택 조사와 실험이 필요하다. 현재 후보를 최종 stack으로 채택하지 않는다.

실제 명령:

```sh
python3 -m py_compile scripts/accessible_performance_probe.py
python3 scripts/accessible_performance_probe.py target/release/dlpad-accessible docs/validation/accessible-performance/current --font /mnt/c/Windows/Fonts/malgun.ttf
python3 scripts/accessible_performance_probe.py target/release/dlpad-accessible docs/validation/accessible-performance/software --font /mnt/c/Windows/Fonts/malgun.ttf --software
python3 scripts/accessible_performance_probe.py target/release/dlpad-accessible docs/validation/accessible-performance/softpipe --font /mnt/c/Windows/Fonts/malgun.ttf --software --software-driver softpipe
```

실행 exit0는 관측·cleanup 완료이며 성능 예산 PASS가 아니다. 앱 로그 각64KiB·10초 CPU tick/RSS 표본·binary64KiB streaming hash·smaps8MiB 입력 상한/상위20개 출력. 10,000목록/실제다운로드/전체tree/GPU별메모리/IME/reader/8시간/다른OS/first-frame는 별도 게이트다. 이후 Pause UI 수정 binary는 새 snapshot이며 이 숫자를 확대 적용하지 않는다.

## 동일 UI CPU renderer 실측 (2026-10-03 KST)

별도 dlpad-cpu SHA256 `28b412773a5fc1434f86d396667a30e7b6d671a8e30affbe1a41cb1ecea7d658`,10,171,256B에서같은Candidate 다운로드UI를재사용했다. [원시결과](accessible-performance/cpu/result.json): 10.044초/RSS50592KiB(49.4MiB)/단일코어idle0%,80MiB제안budget PASS. `python3 scripts/accessible_performance_probe.py target/release/dlpad-cpu docs/validation/accessible-performance/cpu --font /mnt/c/Windows/Fonts/malgun.ttf` exit0. 주담당은CPUidlePNG를직접봐같은한글URL/tool/root/FFmpeg/시작·재개UI를확인했다. ownedGUI/privatebus회수와임시데이터제거완료.

새host는GLcontext를만들지않고egui softwarebackend→softbuffer로pixels를보낸다. top20메모리mapping에GL/LLVM이없으며 `readelf -d target/release/dlpad-cpu`의NEEDED는libgcc_s/libm/libc/ELFloader뿐이었다. 이관측만으로모든OS/driver/GPU전체메모리를검증했다고표현하지않는다. rootCargo에는정확0.0.2renderer/stdonly와기존egui0.33.3/winit0.30.13/softbuffer0.4.8를사용하며새package3개/505827B;[lock독립대조](cpu-lock-review.json)에Windows6dependencyedge변경도기록한다. KMS/DRM기본features와renderer의AccessKit없는helper는사용하지않는다.

동일source의actualGUI4case/IME/private무음Orca는각전용증거를따른다. 이후WindowsactualGUI초기화순서수정은새snapshot이며위비용/firstpaint결과를새hash에자동적용하지않는다. 전체UIparity/10000목록/첫화면/200%/장시간/3OS/숨김40MiBbudget는후속게이트다.
