# startup admission/reaper 최신 release 정상 앱 회귀

2026-10-04 root의 최신 admission/reaper full7 및 release READY 이후 단독 immutable CPU 앱을 실제 private Xvfb GUI로 재검증했다. 기존 harness/source는 수정하지 않았고 이전 a67 release fault-env 증거도 그대로 보존했다. 사용자 desktop/Documents/실제 영상/기존8h/global 설정/다른 unit은 대상으로 하지 않았다.

```text
python3 scripts/user_service_app_probe.py --binary /tmp/dlpad-admission-app-caw5i3fa/dlpad-cpu --expected-sha 97cfbcecf2cc664326adfe37a3d97f92a952b4a3fb2b147ebf5b9b5e0729a7ce --output /mnt/d/project/dlpad/docs/validation/evidence/guardian-startup-wait-app-20261004
```

실제 exit0, active-hidden/complete-hidden/stop-exit 3flow 모두 PASS. [원시 결과](evidence/guardian-startup-wait-app-20261004/result.json), case별 service-proof/result/가상화면 screenshots/bounded logs/합성 SQLite backup, 실행한 probe/background/helper source snapshot과 source SHA를 보존했다. 단독 chmod0500 dlpad-cpu(12,449,656B) 외 helper가 없었고 시작/끝 SHA97cfbcec가 같으며 snapshot directory도 app 하나였다. shared snapshot은 삭제/수정하지 않았다.

backend/debug bus/startup fault env를 테스트 Python process에서 모두 제거하고 옵션 없이 실행했다. 3개 실제 primary environ에서 세 key 부재를 boolean으로 확인했다. secondary는 launch env만 관찰하며 실제 secondary/guardian의 전체 환경 부재 검증으로 확대하지 않는다. 환경 값/자격 증명을 출력하지 않았다.

active-hidden은 same worker heartbeat/PID/start와 same app/job/한 attempt의 숨김→복귀→Cancelled/exit0, complete-hidden은 숨겨진 Downloaded 및 정확 synthetic artifact 확인→원앱복귀/exit0, stop-exit은 close prompt 중단 후 종료로 Cancelled/exit0였다. restored screenshot의 한글 label/최종 파일 확인 완료 표시도 직접 확인했다.

각 worker PPID guardian은 고유 dlpad-job unit의 InvocationID/MainPID/start/cgroup/pidfd로 확인했고 exe dev/inode/SHA가 동일 단독 release 앱이었다. 3개 worker/guardian exact identity 소멸/pidfd ready,3cgroup 제거/service GC, unrelated library sentinel inode/SHA/content 유지 및 외부 owned process 보존/자체 wait를 확인했다. private app/bus/Xvfb/reader cleanup도 true였다. [종료 재검사](evidence/guardian-startup-wait-app-20261004/post-run-verification.json)에서는 총11개 기록 app/worker/guardian PID 경로와3cgroup 부재, 실행 probe source SHA 일치를 재확인했다.

이 자료는 최신 Linux 등록 앱의 기본 정상3flow 회귀다. 다른 OS/실제 media/장기8h/실제 deferred kernel child/manager 재시작 또는 absent-manager 전체 UI를 검증 완료로 표현하지 않는다. 제품 source/Cargo와 전체 P0 통합은 root 담당이다.
