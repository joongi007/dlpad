# S10 통합 및 fixture 수정 후 Windows CPU CLI

2026-10-03 KST. `python3 scripts/windows_native_probe.py {clippy,test,build} --features cpu --seconds 1200`를privateCLI boundedownedJob으로각각실행했다. Clippy6.46초/test15.17초/release30.97초모두exit0. 실제167passed/1ignored/0failed: app15/accessible22/CPU27/core21/engine73/diagnostic3/Windowsprocess6. 진단7개회귀는accessible/CPU양binary에서통과했다.

compiledinput237개각before/after및세작업간동일. runner전후SHA `a665ddae9f74f85d72cc8522a3e67441f58d38695333e6aa7b219dc88a7de55a`, lockSHA `b49000da1a7f0a3b4c4697bceceb76c3d2b7789e36d10e2d678bfe9314ba9031`. inputdesktopunchanged모두확인.

새CPUexe5507584bytes SHA `298e1b7b449dfc1472c4649dfa485736c19088f1dbdc7e30c1243e1dfb00899c`를GUI담당에게READY로전달했다. prebuildLibrarySHA38e7d11...과다르다. 최초S10검증은시험실패로release를실행하지않았기때문에동일S10생산코드의fixture수정전Windowsrelease와비교하는증거는없다. test-onlyfixture수정이runtime행동을변경했다고추론하지않는다.

최초Os183실패는windows-cpu-diagnostics/에별도보존하며이번성공으로삭제/덮어쓰지않았다. 전체argv/inputmanifest/rawlogs/result는windows-cpu-diagnostics-fixturefix/에보존했다. 실제CPUWindows진단GUI/clipboard/AccessKit/IME/성능/MSVC/macOS/remoteCI는이CLI검증에포함하지않는다. fixture수정외제품/Cargo/runner변경없음.
