# 사용자 transient guardian 합성 실험: raw CLI

[작업 계약](../work-items/S03-systemd-user-transient-prototype.md), [실제 result](evidence/systemd-user-transient-20261003/result.json). Proposed capability 실험이며 제품 adapter 변경/채택이 아니다.

`python3 scripts/systemd_user_guardian_probe.py /mnt/d/project/dlpad/docs/validation/evidence/systemd-user-transient-20261003`: exit2. 두 합성 case의 FD/정리 capability는 통과했지만 **literal argv는 실패**하므로 overall_passed=false를 보존했다.

## 실제 scope

고유 dlpad-fixture UUID 사용자 service 두 개만 만들었다. Type=exec/KillMode=control-group/SendSIGKILL=yes/Restart=no/RemainAfterExit=no/TimeoutStopSec=2s/RuntimeMaxSec=45s다. 정확unit/InvocationID/MainPID/start와guardian/setsid child/leaf의동일ControlGroup membership을 확인했다. FD0은실제Unixsocket이며합성guardian의양방향readiness자료를받았다. 기존manager/global설정/다른unit/제품/Cargo/8h시험/desktop을변경하지않았다.

EOF case는소유control끝을닫아guardian이EOF를기록하고자연종료했다. launcherexit0/정리7.39ms. SIGKILL case는다시unit/MainPID/start를대조한guardian exactpidfd만SIGKILL했고manager가setsid자손도정리했다. launcherexit255/정리6.28ms. 두case의6개originalprocessidentity가모두사라졌고각pidfd는ready이며cgroup도제거됐다. 외부소유sentinel은두case동안동일identity로살아있었고최종Popen종료·회수했다. 새소유/tmp는제거했고실행직후result/log5개파일총21639byte였으며후속조사전당시probe-source.py사본을추가보존했다. 정확PID/InvocationID/명령·실행당sourceSHA는result와caseJSON에있다. 숫자PID추측·무관process종료는없다.

## 실패한 literal argv

합성환경 DLPAD_FIXTURE_VAR=expanded sentinel을명시했다. raw `$DLPAD_FIXTURE_VAR`는expanded와sentinel 두argv로쪼개졌고`${DLPAD_FIXTURE_VAR}`는expanded sentinel로치환됐으며prefix형태도치환됐다. 한글/공백/따옴표/%n은이표본에서보존됐다. 두case모두동일실패다. 이는shell을호출하지않더라도v249 manager의환경확장이제품literalargv를손상시키는실제증거다. 이CLIwrapper를그대로채택하지않는다. raw실패자료는덮어쓰지않고v249ExecStartEx/no-env-expand후속을별도case로조사한다.

## 한계와 다음 단계

이실험의guardian은Python합성worker이며제품headlessguardian 자체가아니다. guardian자체subreaper/PDEATHSIG·pause/DB/artifact·동시app+guardian죽음·user managerfailure·두job동시실행·악의적cgroup탈출·3OS·8h를검증하지않았다. 실제productionintegration에는소유unit/cgroup 확인,endpoint독점/readiness,정상정리ack,실패와모호한권한에failclosed,정확literalargv와boundedIO비용검증이필요하다. 성공한capability를전체R08완료로확대하지않는다.

후속v249ExecStartEx/no-env-expand 실제결과는 [별도검증](systemd-execstartex-prototype.md)에기록한다. rawCLI실패를삭제하거나통과로고치지않았다.
