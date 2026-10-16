# CPU 테마·배율 통합 Windows GNU CLI

2026-10-03 KST. 기존projectnative runner를읽기전용으로clippy/test/build각각 `--features cpu --seconds 1200` 실행했다. Windows전용cargo-home/target/tmp/privateCLI boundedJob사용. Clippy7.63초/test25.80초/release36.66초모두exit0, actual188passed/1ignored/0failed였다.

시험개수는app15/accessible31/CPU36/core21/engine76/diagnostic3/Windowsprocess6. Appearance9개Windows시험은accessible/CPU양binary에서통과했고Unix조건부permission/symlink2개는실행되지않았다. engine readonlyStore common3개는Unicodebytebound/TooBig/값없음·손상·DB생성/migration없음/EXCLUSIVEbusy실패·기존선택보존을실제검증했다. finalsymlinkNOFOLLOW시험은cfgUnix라Windows에없다. Windowsreparse/NOFOLLOW실제거절성공으로확대하지않는다.

compiledinput238개각before/after와3작업간동일. runner시작/종료SHA `a665ddae9f74f85d72cc8522a3e67441f58d38695333e6aa7b219dc88a7de55a`, Cargo.lockSHA `b49000da1a7f0a3b4c4697bceceb76c3d2b7789e36d10e2d678bfe9314ba9031`. 모든로그inputDesktopunchanged확인.

새 `.local-tools/windows/target/release/dlpad-cpu.exe`5555200bytes SHA `dd0241cdab5a6728be8593ad07087397554908128e283e952a48f188c1879a99` READY를root에게전달했다. 전체argv/source/vendor/hash/raw시험/build로그는windows-cpu-appearance/에보존했다. 제품/Cargo/runner수정·GUI/clipboard접근없음.

미검증: 실제Windowstheme/scale/SystemOSchange/IME/AccessKit/firstpaintcost, WindowsNOFOLLOWfinalsymlink, MSVC/macOS/remoteCI. 모듈논리Systemevent시험을actualOSevent로대체하지않는다.
