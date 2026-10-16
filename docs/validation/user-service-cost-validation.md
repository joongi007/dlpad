# 기본 user-service CPU 앱 비용 표본

2026-10-03 Linux private Xvfb native GUI. 단독 immutable CPU SHA `ea75955c3335612097b39cf7d9587c3fd1dc13f07ca3bc6ac854b93de5e596cc`, 실행파일12,438,904B(약11.86MiB). 별도 helper 파일 없이 같은 executable이 app/launcher/guardian 진입점을 제공한다. root가 같은copy로 병렬검증했으며 이 collector는 읽기만 사용하고 삭제하지 않았다.

## 실제 두 표본

| 표본·단계 | 프로세스 | 실제 표본초 | 최대 RSS MiB | 단일코어 CPU % |
| --- | --- | ---: | ---: | ---: |
| 1 · idle | app | 9.865 | 55.480 | 0.5068 |
| 1 · active | app | 9.908 | 58.957 | 1.4130 |
| 1 · active | launcher | 9.908 | 8.129 | 0.2019 |
| 1 · active | guardian | 9.908 | 6.660 | 0.0000 |
| 1 · active | worker | 9.908 | 10.422 | 0.1009 |
| 2 · idle | app | 9.976 | 54.023 | 0.7017 |
| 2 · active | app | 9.869 | 57.812 | 1.7226 |
| 2 · active | launcher | 9.869 | 6.789 | 0.3040 |
| 2 · active | guardian | 9.869 | 5.145 | 0.2027 |
| 2 · active | worker | 9.869 | 10.410 | 0.1013 |

warmup2초 뒤 idle10초, 실제 합성 다운로드 worker/journal 생성과 소유 unit 확인 뒤 active10초를 관측했다. /proc stat의 utime+stime만 사용하고 자식 CPU를 더하지 않았다. SC_CLK_TCK/monotonic 실제 첫·마지막 표본 간격으로 계산했다. 각 단계98~99개 표본이며 최대10Hz/전체2000행 상한 안이다. guardian의 표본1 CPU0.0000%는 해당 구간에 tick 증분을 관측하지 못한 값이며 CPU 비용이 절대 없다는 뜻이 아니다. 약10초/100Hz tick에서 한tick은 약0.1%로 양자화된다.

독립 재계산에서 모든 CPU/RSS 값이 원 collector와 일치했다. 같은 수집 cycle(프로세스 간 시간차0.1초 이하)의 app+launcher+guardian RSS 최대 합은 표본1 75,516KiB(73.746MiB), 표본2 71,420KiB(69.746MiB)였다. CPU 근사합은 각각1.6149%/2.2292%다. RSS는 공유 executable/library 페이지를 중복 합산할 수 있으므로 고유 물리메모리 사용량(PSS/USS)으로 쓰지 않는다. 개별 perPID HWM은 raw result.json에 보존했다.

worker는 synthetic-tool Python fixture이며 실제 영상 다운로드·FFmpeg 처리 비용이 아니다. worker는 infrastructure 합계에서 제외했다. 동일한 앱의 hide/restore 이후에도 같은worker/job·한attempt가 유지되고, 실제 중단 후 정상 앱 exit0/Cancelled 상태가 확인된 workload다. hidden10초 비용·실제 media 처리·최대 보고서·라이브러리 peak·8h 누수 표본은 아니다.

## idle helper 및 공유 runtime 구분

표본1은 idle app만 등록해 launcher/guardian을 측정하지 않았다. raw present=false를0MiB/0CPU 측정으로 해석하지 않는다. 이를 명확히 하려고 표본2에서는 idle98개 시점에 소유 app의 모든 thread children을 최대128개 범위로 확인했고 child/guardian marker가 모두0이었다. 활성 job이 없는 앱 idle 구간의 owned launcher 부재 표본이다. manager guardian은 해당job의 launcher/control 계약으로 연결되므로 idle에 등록된 guardian PID가 없었으나 공유 user-manager 전체의 service 부재를 주장하지 않는다. 10Hz 관측은 표본 사이의 매우 짧은 프로세스를 포착했다는 증거가 아니다.

host systemd/user-manager와 libsystemd.so.0는 기존 OS runtime이다. manager RSS/CPU나 시스템전체 비용을 앱 소유 프로세스 합계에 넣지 않았다. launcher가 기존 runtime library를 동적으로 사용하며 그 mapped page의 RSS는 소유 launcher RSS에 반영될 수 있다. 이 library를 새로 설치하거나 전역 manager/config/security를 변경하지 않았다. 다른OS/manager없는환경/배포 패키지 전체크기는 미검증이다.

단독 executable은 이전166946 paged baseline12,402,040B보다36,864B 크다. 그 사이 설정스크롤·확장metadata·backend 통합이 함께 들어갔으므로 차이를 user-service 한 기능에만 귀속하지 않는다. 폰트·관리도구·OS runtime을 포함한 전체 배포파일 크기로 확대하지 않는다. 두10초 표본의 변동을 확정적인 회귀/개선으로 판단하지 않는다. host load before/after는 각 raw에 기록했고 부모의 병렬검증과 장기시험이 있을 수 있다. 앱 perPID CPU와 host전체 load를 혼합하지 않았다.

## 명령·독립 종료 검증

`python3 scripts/user_service_cost_probe.py --binary /tmp/dlpad-user-service-app-vxa0u746/dlpad-cpu --expected-sha ea75955c3335612097b39cf7d9587c3fd1dc13f07ca3bc6ac854b93de5e596cc --output /mnt/d/project/dlpad/docs/validation/evidence/user-service-cost/current-1` 및 동일인자의current-2를 실행했다. 둘다 exit0/native service proof PASS다. current-2는 idle 자식관측과180초 전체deadline을 추가했다. 원시 current-1을 덮어쓰지 않았다.

[evidence/user-service-cost/](evidence/user-service-cost/)에 각 raw/GUI PNG/SQLite/service-proof 및 independent-verification을 보존했다. 새로운 독립 Python process에서 기존harness를 readonly import하고 해당 process의 module 변수만 계측으로 교체했다. 기존소스·기존harness·Cargo를 수정하지 않았고 stdlib global을 바꾸지 않았다. 사용자 desktop/clipboard/실제 media/계정은 사용하지 않았다.

worker PPID, 고유unit InvocationID/MainPID/start/cgroup과 guardian executable dev/inode/SHA를 확인했다. launcher는 소유 app의 thread children과 정확 marker·같은 executable으로 식별했다. 기존8h PID/unit은 대상으로 삼지 않았다. host /proc에서 두run의 app/launcher/guardian/worker8개 원본PID/start 부재, 두runtime prefix 잔존0, temp폴더 회수, 고유unitLoadState=not-found/MainPID0/빈InvocationID/cgroup삭제를 직접 확인했다. SQLite integrity=ok, 최종Cancelled/rev4/한attempt, sentinel보존, binary SHA/단독artifact 유지도 독립 확인했다.

Python 구문 검사와 문서 검사 exit0. 이 제한 Linux 비용 표본은 S00 전체 스택채택/P0 완료/Windows/macOS/숨김40MiB/시작1초/실제media 비용/음성·전체reader 검증을 대체하지 않는다.

## 최신 startup cleanup a67d722 재검증

root의 alias FD/EP 정리 후 immutable `/tmp/dlpad-startup-cleanup-app-n8emvns9/dlpad-cpu` SHA `a67d7224f3b706547d903fc0058d2a6200efd98609ff906243a2f6a19d21c1a7` /12,439,160B를 같은 전용 collector로 fresh current-3에 실행했다. 앞의ea759 두표본은 불변 보존했다. executable 크기는256B 증가했다.

| 단계 | 프로세스 | 표본초 | 최대 RSS MiB | 단일코어 CPU % |
| --- | --- | ---: | ---: | ---: |
| idle | app | 9.962 | 54.219 | 0.7027 |
| active | app | 9.883 | 57.793 | 1.7202 |
| active | launcher | 9.883 | 7.406 | 0.2024 |
| active | guardian | 9.883 | 5.559 | 0.1012 |
| active | worker | 9.883 | 10.465 | 0.1012 |

idle 소유 app thread-children 관측99개에서 children/guardian marker0이었다. active infrastructure 동일cycle RSS합은72,456KiB(70.758MiB), CPU근사합2.0237%다. 앞의공유RSS/짧은CPU표본 제한을 동일하게 적용한다. FD alias 추가가 비용이나cleanup에 전혀 영향 없음을 일반화하지 않고 실제 이표본의값을 보고한다.

`python3 scripts/user_service_cost_probe.py --binary /tmp/dlpad-startup-cleanup-app-n8emvns9/dlpad-cpu --expected-sha a67d7224f3b706547d903fc0058d2a6200efd98609ff906243a2f6a19d21c1a7 --output /mnt/d/project/dlpad/docs/validation/evidence/user-service-cost/current-3` → exit0/native active-hidden service proof PASS. source/Cargo/sharedharness 수정은 없다.

app·launcher·guardian의 동일exe와 syntheticworker를 포함한 모든 역할의 PID/start 인수와 기존 native hide/restore/중단 흐름이 통과했다. host독립재계산/네PID원본 부재/runtime잔존0/temp회수/unitnot-found/cgroupgone/DBintegrityok/Cancelled한attempt/binary SHA·단독artifact 유지도 PASS다. GUI/collector는 모두terminal이며 공유copy는 삭제하지 않았다. alias FD의 모든번호·실패경로·Windows런타임을 이비용시험으로 대신하지 않는다.

Python 구문 검사와 문서 검사 exit0. 최신snapshot에서 비용과 기본 native lifecycle의제한 Linux 재검증을 완료했으며 앞서명시한전체OS/전체P0/8h/실제media/hidden비용의미검증은 남는다.

## 97cfbcec startup reaper/admission 표본

root측정current-5에서idle최대RSS54.617MiB/CPU0.6116%,activeapp58.465MiB·launcher7.523MiB·guardian5.586MiB였다. 개별최대RSS합71.574MiB는공유page중복과서로다른최대시점을포함할수있다. [원시결과](evidence/user-service-cost/current-5/result.json), [rootCPU/RSS재계산·4PID/소유cgroup부재](evidence/user-service-cost/current-5/root-review.json). 실제nativeactive-hidden정상종료도PASS. 최신실행파일12449656B이며이전12439160B대비10496B증가했다. 숨김peak/실제media/8h/다른OS비용은별도게이트다.

current-4는공용app_probe release_fault_env와costnamespace의호환오류로GUI시작전exit1/0samples였다. raw실패를보존하고costnamespace에None을추가한뒤current-5로재검증했다. 0samples를0RSS로해석하지않는다.
