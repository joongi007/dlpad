# CPU feature 실제 Windows GNU CLI 검증

2026-10-03 KST. 기존 `scripts/windows_native_probe.py`를 수정 없이 private CLI desktop의 bounded owned Job으로 실행했다. Windows 전용 Cargo home/target/tmp를 사용했다. 초기 Clippy33.41초/test40.10초/release60.55초 모두exit0였다. 초기실행의 runner 시작SHA가 별도로 보존되지 않았으므로 캐시된 동일 검사를 반복하여 원시 증거를 보완했다.

반복 명령은 `python3 scripts/windows_native_probe.py {clippy,test,build} --features cpu --seconds 1200`이다. Clippy8.57초/test11.18초/release5.50초 모두exit0,131passed/1ignored/0failed였다. source235개의 각before/after와 세작업간 입력이 동일하다. runner SHA 시작/종료 모두 `a665ddae9f74f85d72cc8522a3e67441f58d38695333e6aa7b219dc88a7de55a`, Cargo.lock SHA `b49000da1a7f0a3b4c4697bceceb76c3d2b7789e36d10e2d678bfe9314ba9031`이다. 모든로그에서 input desktop unchanged가 확인됐다.

시험수: 기본app15/accessible5/CPU10/core21/engine71(ignored1)/diagnostic3/Windowsprocess6. CPU10에는 기존Candidate5와 신규host5가 포함된다. 실제 host 회귀는 zero-size/frame budget, deadline wake coalescing, malformed partial texture, hidden fonttexture 적용 후 visible partialupdate, malformed meshindex다. 실제 창·reader·IME 동작은 단위시험으로 대체하지 않는다.

release `dlpad-cpu.exe`5296640bytes SHA256 `82602ea20f183bdf44021e60ad26416703c60dd5f9c6e89ec7bb805aee1f8704`. 바이너리 GUI 실행은 하지 않았다. 정확 argv/전체 inputmanifest/초기 및 반복logs/기타exeSHA는 windows-cpu-feature/result.json과 같은 폴더에 보존했다.

미검증: 실제CPU Windows private GUI/AccessKit tree·screenreader·IME·clipboard·성능, MSVC, macOS, 원격 CI. cfgLinux 전용 disk/media/guardian/soak는Windows0tests로 완료범위에 포함하지 않는다. 이번 결과는 Windows GNU CPU feature의 실제 compile/lint/unit/integration CLI 및 release 검증이다.
