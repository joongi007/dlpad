# 등록 CPU 앱의 기본 user-service 네이티브 검증

2026-10-03 Linux/WSL boot `0bc0c937-eee9-4d90-95dc-31a2122b6352`에서 root가 채택한 등록 앱 기본 user-systemd 동작을 실제 단독 CPU release GUI로 검증했다. explicit backend env를 사용했던 이전 engine13case와 구분된다. 사용자 desktop에는 창을 띄우지 않았고 기존 private Xvfb/접근성 D-Bus 합성 harness를 읽기 전용 import로 재사용했다. 실제 media/사용자 Documents를 사용하지 않았다.

## 실제 실행

root release READY 이후 실행했다.

```text
python3 scripts/user_service_app_probe.py --binary /tmp/dlpad-user-service-app-vxa0u746/dlpad-cpu --expected-sha ea75955c3335612097b39cf7d9587c3fd1dc13f07ca3bc6ac854b93de5e596cc --output /mnt/d/project/dlpad/docs/validation/evidence/user-service-app-default-20261003
```

실제 exit0, 3flow PASS. [통합 결과](evidence/user-service-app-default-20261003/result.json), case별 result/service-proof JSON, bounded app/bus 로그, screenshot 및 최대2MiB 합성 SQLite backup을 보존했다. 실행한 probe/background/helper source snapshot과 [관련 source SHA](evidence/user-service-app-default-20261003/source-hashes.json)도 기록했다. 공유 product/Cargo/기존 probe는 수정하지 않았다.

## 기본 보호 계약

snapshot directory에는 chmod0500 `dlpad-cpu` 한 파일(12,438,904B)만 있었다. adjacent dlpad-process-guard helper가 없는 상태다. 시작/끝 SHA256 `ea75955c3335612097b39cf7d9587c3fd1dc13f07ca3bc6ac854b93de5e596cc` 동일, 마지막 directory contents도 dlpad-cpu 하나였다. shared snapshot은 삭제/수정하지 않았다.

시험 Python process에서 DLPAD_GUARDIAN_BACKEND/DLPAD_TEST_SYSTEMD_BUS_PATH를 unset했다. 각 앱 launch env에서 부재를 기록했고 primary 실제 `/proc/PID/environ`에서도 해당 key 부재를 boolean으로 확인했다. 환경 전체 값이나 실제 자격 증명을 증거에 출력하지 않았다. secondary 복귀 launch는 launch env 부재를 확인했으며 별도의 secondary 실제 environ 조회를 주장하지 않는다.

각 active download의 worker PPID를 guardian으로 관찰했다. guardian은 고유 dlpad-job nonce service의 MainPID이며 InvocationID/PID/start/cgroup을 확인한 뒤 pidfd를 열었다. worker와 guardian은 같은 user-manager cgroup이고 guardian `/proc/PID/exe` dev/inode 및 SHA가 단독 app snapshot과 정확히 같았다. 따라서 다른 adjacent helper나 direct backend를 사용한 결과가 아니다. user manager는 고정 `/run/user/UID/bus`, GUI 접근성/activation은 private fixture runtime/bus를 사용했다.

| flow | 종료 job state | 숨김 / 원앱 복귀 | 결과 |
| --- | --- | --- | --- |
| active-hidden | Cancelled | True / True | PASS |
| complete-hidden | Downloaded | True / True | PASS |
| stop-exit | Cancelled | False / False | PASS |

active-hidden은 같은 worker heartbeat 증가, 같은 PID/start와 같은 primary 앱/작업/한 attempt로 숨김→secondary launch 복귀를 확인한 뒤 중단 후 종료했다. complete-hidden은 숨겨진 상태에서 Downloaded가 되었고 same app 복귀 및 실제 합성 artifact bytes를 확인했다. stop-exit은 활성 다운로드 중 native close prompt의 중단 후 종료를 선택해 앱 exit0/Cancelled journal을 확인했다. 복귀 screenshot의 한글 label과 최종 파일 확인 상태도 직접 확인했다. primary/secondary 모두 실제 가상 화면에만 실행했다.

3개 worker와 guardian 원래 identity가 zombie까지 남지 않고 사라졌고 guardian pidfd 모두 ready였다. 3개 cgroup 제거/고유 service not-found GC를 확인했다. [추가 재검사](evidence/user-service-app-default-20261003/post-run-verification.json)에서는 primary3/secondary2/worker3/guardian3 총11개 기록 PID 경로 부재와 cgroup 부재를 확인했다. 각 case의 unrelated library sentinel 파일은 inode/SHA/contents를 temporary cleanup 직전 유지했으며, 외부 소유 synthetic process도 원래 identity로 살아 있었다. 끝에 자체 process를 종료/wait했다. 기존 harness의 app/private-bus/Xvfb 및 bounded-reader cleanup은 각 case owned_cleanup=true였다.

## 한계와 인수 범위

잘못된 immutable SHA 입구는 실제 exit2로 GUI/service/output 생성 전에 거절했다. 이 host의 등록 Linux CPU 앱 기본 backend와 3개 사용자 흐름을 검증했다. 실제 media, 다른 OS, absent manager UI/진단, manager 재시작/race, 등록 앱 direct override 거절 GUI, 장기 user-service 8h soak는 이 자료만으로 검증 완료라고 하지 않는다. 기존 engine13case 또는 이전 immutable direct-guardian 8h의 결과와 별개다. 기본 backend/ADR 최종 결정 및 P0 상태는 root 담당이다.
