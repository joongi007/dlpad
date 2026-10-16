# S03: v249 ExecStartEx literal argv 합성 후속

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: soak_recovery_audit; scripts/systemd_user_guardian_probe.py, 이 workitem, docs/validation/systemd-execstartex-prototype.md, 전용 evidence/systemd-execstartex-20261003/
- 선행 작업: S03-systemd-user-transient-prototype raw CLI literal 실패

## 사용자 결과

실패한rawCLI자료를보존하고공식v249D-Bus ExecStartEx/no-env-expand가FD0·literalargv·소유cgroup정리계약을지키는지별도실험한다.

## 계약과 범위

공식v249 dbus-service/dbus-execute/exec-util/sd-bus source를먼저확인한다. Pythonctypes로기존libsystemd를직접호출하며추가dependency/Cargo/schema없다. 고유transientunit mode=fail, 정확InvocationID/MainPID/start/cgroup/pidfd,외부sentinel과cleanup을유지한다. shellescape/colonprefix미검증우회나기존unit/8h/global설정변경은없다. 제품통합은root의후속담당이다.

## 인수 조건

ExecStartEx a(sasas)/no-env-expand 및stdFD0를실제로검증한다. 불가지원/권한/argv실패를그대로기록하며raw실패를덮어쓰지않는다. 소유fixture만정리하고bounded자료를새evidence에보존한다.

## 검증과 handoff

공식v249ExecStartEx/stdFD/flag지원확인후Pythonctypes로기존libsystemd직접호출실험했다. --transport execstartex 실제2caseexit0: FD0/readiness/EOF/guardianSIGKILL cgroup정리 및7개literalargv PASS. 50.35/53.61ms관측(50ms폴링포함), exact6identity부재/pidfdready/cgroupremoved,외부sentinel보존+자체cleanup,ownedtmp제거. rawCLIargv실패원본과당시source사본보존. 실제제품통합·전체R08완료와구분한다. 정확근거/미검증은전용validation/evidence에기록했다.
