# v249 ExecStartEx/no-env-expand 실제 합성 후속

[작업 계약](../work-items/S03-systemd-execstartex-prototype.md), [실제 결과](evidence/systemd-execstartex-20261003/result.json), [공식 source·실행 library 근거](evidence/systemd-execstartex-20261003/source-review.json). 앞선 [raw CLI argv 실패](systemd-user-transient-prototype.md)를 보존한 별도실험이다. Proposed 후보이며제품adapter/아키텍처채택은없다.

## 실행과 결과

`python3 scripts/systemd_user_guardian_probe.py /mnt/d/project/dlpad/docs/validation/evidence/systemd-execstartex-20261003 --transport execstartex`: exit0. FD/정리capability=true/literalargv=true/overall=true다. Python stdlib ctypes로이미설치된libsystemd.so.0을직접호출했다. 추가Python/Cargo의존성·제품sharedsource/schema변경은없다.

고유사용자 dlpad-fixture UUID 서비스2개를 StartTransientUnit mode=fail로만들었다. interactive authorization=false이며시스템manager/설정변경은없다. Type=exec/KillMode=control-group/SendSIGKILL=yes/Restart=no/RemainAfterExit=no/TimeoutStopUSec=2000000/RuntimeMaxUSec=45000000/CollectMode=inactive-or-failed다. StandardInputFileDescriptor(h)에소유socketpair의guardian끝을전달하고stdout/stderr에는소유boundedlogFD만전달했다.

| case | FD/literal 결과 | 실제 정리 관측 |
|---|---|---|
| EOF | FD0 Unixsocket/readiness/owner끝닫기→EOF기록, 7개argv정확 | 50.35ms, guardian/setsid child/leaforiginalidentity모두부재, pidfd모두ready, cgroup제거 |
| guardian SIGKILL | unit/InvocationID/MainPID/start를다시검증한exactguardianpidfd만SIGKILL, 7개argv정확 | 53.61ms, 동일3identity부재/pidfdready/cgroup제거 |

정리시간은50ms polling을포함한이번관측표본이며production의즉시정리상한을확정하지않는다. directDBuscase에는launcher Child/exitcode가없다. fd readiness·exactprocess부재·cgroup제거·EOF파일을별도로확인했으며반환0을제품의도구성공이나실제guardian exitcode로간주하지않는다. 외부소유sentinel은두case동안동일identity로생존했고최종Popen의소유process만terminate/wait했다. 새소유/tmp는제거했다. 다른unit/기존8hPID/관리자/global설정/desktop변경은없다.

## 공식 v249 지원과 literal argv

실험전에공식v249 dbus-service.c에서ExecStartEx와stdFD transientproperty를확인했다. dbus-execute.c는Ex variant의a(sasas)를읽고flag문자열을해석하며, exec-util.c의no-env-expand는NO_ENV_EXPAND에연결된다. 따라서존재하지않는신규version옵션이나colonprefix/shellescape를추측해사용하지않았다. [서비스 D-Bus](https://raw.githubusercontent.com/systemd/systemd/v249/src/core/dbus-service.c), [Ex setter](https://raw.githubusercontent.com/systemd/systemd/v249/src/core/dbus-execute.c), [flag mapping](https://raw.githubusercontent.com/systemd/systemd/v249/src/shared/exec-util.c).

실제DLPAD_FIXTURE_VAR=expanded sentinel이서비스환경에있는데도 `$DLPAD_FIXTURE_VAR`, `${DLPAD_FIXTURE_VAR}`, `prefix-${DLPAD_FIXTURE_VAR}`, 공백값, 한글, `%n`, 따옴표값이입력과동일한7개argv로도착했다. raw CLI에서는첫값이두argv로분해되고다음두값도치환됐던동일표본이다. no-env-expand가replace_env_argv를우회하는공식계약과실제결과가일치한다. [실행 source](https://raw.githubusercontent.com/systemd/systemd/v249/src/core/execute.c).

raw CLI 원시result/caseJSON/log는덮어쓰지않았고당시probe-source.py사본도보존했다. 후속실행당시source와libsystemd의SHA는source-review/result에있다. 서로다른sourceSHA의실험을동일snapshot검증이라고표현하지않는다.

## 남은 P0 계약

성공은사용자권한의고유transient서비스에서합성Python guardian/setsid자손을정리하고literalargv표본을보존한범위다. 실제제품headlessguardian의R/T/K protocol, CString/invalidbyte/NUL경계,환경최소전달·credentials보호,서버미지원/권한/exec/readiness실패,두job동시정리·무관작업보존,app+guardian동시SIGKILL,manager실패/로그아웃·절전·악의적cgroup탈출·취소/DB/publication/actualmedia/8h는미검증이다.

root가공유Rust엔진의DBus라이브러리호출·ownedidentity·출력FD·완료/정리ack·boundedtimeout·failclosed를조정해야한다. Python prototype을제품runtime으로넣거나현재Linux전체guardian고장보호가완료됐다고보고하지않는다. 연구Prototype의부분성공은전체R08/S03완료가아니다.

## 검증과 비용

syntax확인 `python3 -m py_compile scripts/systemd_user_guardian_probe.py` exit0, 실제2caseexit0. 로그는case당1MiB상한,JSON도1MiB상한이며고정2case·3ownedPID·상한64KiBreadiness/선택unitproperty만사용했다. 실행직후rawresult/case/log합계20913byte이며source사본/primary목록은별도bounded추가다. 현재manager재사용후보이고extraCargodependency는0이지만제품채택시libsystemd동적runtime조건·manager/per-job비용·배포OScoverage는별도측정해야한다.
