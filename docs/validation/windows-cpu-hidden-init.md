# 숨긴 창 초기화 수정 후 Windows CPU CLI

2026-10-03 KST. 기존 runner를 읽기 전용으로 `clippy`, `test`, `build` 각각 `--features cpu --seconds 1200`으로 실행했다. 실제 Windows GNU private CLI 검사 모두exit0, 경과5.72/13.69/15.26초,131passed/1ignored. source235개 각각before/after 및 세작업간동일. runner전후 SHA `a665ddae9f74f85d72cc8522a3e67441f58d38695333e6aa7b219dc88a7de55a`, lockSHA `b49000da1a7f0a3b4c4697bceceb76c3d2b7789e36d10e2d678bfe9314ba9031`. 모든검사 input desktop unchanged.

새 `dlpad-cpu.exe`5297664bytes, SHA `5cb1a363223ae5a958737c537f6b50466659e44909419b0ba9242159623b37f1`를 통합 담당과 privateGUI 담당에게READY로전달했다. 실제 GUI는 이작업에서 실행하지 않았다. 이전 visible-first AccessKit panic 증거를 보존하며 새CLI성공을 실제GUI수정완료로확대하지않는다. WindowsGNU CLI범위이며MSVC/macOS/CPUreader·IME·성능 미검증.

전체입력·argv·logs는 windows-cpu-hidden-init/result.json과 같은폴더에 보존했다. Cargo/제품/runner는수정하지않았다.

독립 재검토: 현재235개 source mismatch0, runner/lock/exeSHA 일치, raw test result16줄 합산131passed/1ignored/0failed. 세 manifest before/after 동일을 다시 확인했다. 원시 검토는 windows-cpu-hidden-init/independent-review.json.
