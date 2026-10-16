# 최신 startup cleanup release 앱 네이티브 회귀

2026-10-03 root가 최신 alias/EP source의 workspace 및 Windows checks를 마친 뒤 새 단독 CPU release snapshot으로 실제 기본 service GUI를 재검증했다. 검증 범위는 이 Linux host의 isolated Xvfb/private accessibility D-Bus와 synthetic media/temporary library다. 사용자 desktop/Documents/실제 영상/기존 8h/다른 unit/global 설정을 대상으로 하지 않았다.

## 실제 명령과 원시

```text
python3 scripts/user_service_app_probe.py --binary /tmp/dlpad-startup-cleanup-app-n8emvns9/dlpad-cpu --expected-sha a67d7224f3b706547d903fc0058d2a6200efd98609ff906243a2f6a19d21c1a7 --output /mnt/d/project/dlpad/docs/validation/evidence/user-service-app-cleanup-20261003 --release-fault-env after-start
```

실제 exit0/PASS, active-hidden/complete-hidden/stop-exit 3flow 모두 통과했다. [전체 원시](evidence/user-service-app-cleanup-20261003/result.json), 각 case service-proof/result, bounded app/bus/Xvfb logs, native screenshots/합성 SQLite backup, 실행된 probe/background/helper source 및 [source SHA](evidence/user-service-app-cleanup-20261003/source-hashes.json)를 보존했다. 이전 GUI default/strict13/startup4 증거/source는 덮어쓰지 않았다. 공유 product/Cargo와 기존 imported probe는 수정하지 않았다.

snapshot directory에는 chmod0500 dlpad-cpu 한 파일(12,439,160B)만 있고 adjacent helper가 없었다. 시작/끝 SHA `a67d7224f3b706547d903fc0058d2a6200efd98609ff906243a2f6a19d21c1a7`가 동일했고 마지막에도 단독 파일이었다. snapshot은 읽기 전용으로 유지했다.

## release가 debug fault 환경을 무시함

전용 option으로 테스트 Python/child env에 DLPAD_TEST_SYSTEMD_STARTUP_FAULT=after-start를 설정했다. 3개 primary 실제 `/proc/PID/environ`과 3개 실제 service guardian environ에 해당 key가 존재함을 boolean으로 확인했다. 다른 backend/debug bus override는 계속 unset하고 primary 실제 environ 부재를 확인했다. 환경 전체/자격 증명을 출력하지 않았다. secondary는 launch env만 기록하며 별도 actual environ 검증을 주장하지 않는다.

debug에서 실제 accepted-service 실패를 일으켰던 같은 after-start 환경이 있는 상태인데도 release는 세 flow의 다운로드/숨김/중단/cleanup을 정상 수행했다. service guardian exe dev/inode와 SHA는 모두 동일 단독 release snapshot이다. 따라서 debug helper가 아닌 실제 release 프로세스에서 fault env 무시를 확인했다. 다른 debug hook 값이나 release missing-manager 전체 UI 안내를 이 자료로 검증 완료라고 하지 않는다.

## 실제 사용자 흐름과 정리

active-hidden은 같은 worker heartbeat 증가/PID/start, 같은 primary app/job/한 attempt를 유지한 채 숨김→secondary launch 복귀→Cancelled/primary exit0였다. complete-hidden은 숨긴 상태 Downloaded와 정확 synthetic artifact bytes를 확인하고 동일 앱 복귀 후 exit0였다. stop-exit은 native close prompt의 중단 후 종료로 Cancelled/exit0를 확인했다. 복귀 screenshot의 한글 label/최종 파일 확인 완료 표시를 직접 확인했다.

각 download의 고유 dlpad-job unit InvocationID/MainPID/start/cgroup과 worker membership을 확인한 뒤 guardian pidfd를 열었다. 같은 release guardian/worker exact identity 종료, pidfd ready, 3cgroup 제거와 unit not-found GC를 확인했다. [종료 후 재검사](evidence/user-service-app-cleanup-20261003/post-run-verification.json)는 primary3/secondary2/worker3/guardian3 총11개 기록 PID 경로와3cgroup 부재, 실행 probe SHA 일치도 확인했다. unrelated library sentinel의 inode/SHA/contents를 temporary cleanup 전 보존했고 외부 소유 process sentinel도 동일 identity로 살아 있었다가 끝에 자체 종료/wait했다. 기존 harness의 private app/bus/Xvfb/reader cleanup도 전부 true였다.

전체 P0/다른 OS/실제 media/새 backend의 장기8h/manager 재시작 또는 unit race/전체 absent-manager 안내는 root의 후속 통합 인수 범위다. 본 작업은 release default service와 현재 오류 cleanup 변경에 대한 실제 native GUI 회귀다.
