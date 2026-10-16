# 영구 소유 soak 증거 collector

[작업 계약](../work-items/S04-soak-evidence-archive.md). scripts/soak_evidence_archive.py는 합성 headless 시험의 고정 status/log/result만 새 project evidence 폴더에 보관한다. 실제 soak를 시작하거나 프로세스에 signal을 보내지 않는다. 외부 패키지·Cargo 의존성 추가가 없다.

## root의 실행 계약

먼저 root가 새 합성 download/guardian 실행을 준비하고 현재 boot와 정확 PID/start를 확인한다. launch source/binary/helper manifest는 collector destination 밖의 별도 launch 증거에 보관한다. collector destination은 없던 docs/validation/evidence 하위 디렉터리여야 하며 부모가 이미 존재해야 한다. 임시 root는 직접 /tmp 아래 새 폴더다. download runner는 download 이름 뒤 -runner를 붙인 형제 폴더다.

ownership JSON은 정확히 boot_id, roots, processes 세 key다. roots는 download/runner/guardian의 명시적 절대 경로 세 개이고 CLI와 동일해야 한다. processes는 root가 확인한 role→{pid,start_ticks}다. 허용 role은 download_runner/download_owner/download_guardian/download_worker 및 guardian_owner/guardian_guardian/guardian_root/guardian_middle/guardian_leaf/guardian_detached다. 다른 role은 fail-closed다. 모든 알려진 시험 프로세스를 포함해야 하며 collector가 임의로 자손을 찾거나 소유권을 추측하지 않는다.

```json
{
  "boot_id": "00000000-0000-0000-0000-000000000000",
  "roots": {
    "download": "/tmp/dlpad-download-UNIQUE",
    "runner": "/tmp/dlpad-download-UNIQUE-runner",
    "guardian": "/tmp/dlpad-guardian-UNIQUE"
  },
  "processes": {
    "download_owner": {"pid": 123, "start_ticks": 456},
    "guardian_owner": {"pid": 789, "start_ticks": 1011}
  }
}
```

위 값은 형식 예시이며 실행 가능한 실제 identity가 아니다. 실제 root가 만든 manifest로 아래 명령을 수행한다. 이번 작업에서 실행하지 않았다.

```sh
python3 scripts/soak_evidence_archive.py --ownership /tmp/dlpad-soak-ownership-UNIQUE.json --download-root /tmp/dlpad-download-UNIQUE --guardian-root /tmp/dlpad-guardian-UNIQUE --destination /mnt/d/project/dlpad/docs/validation/evidence/soak-archive-UNIQUE --interval 60
```

--once는 한 번 관측 후 종료하며 watching의 exit0은 8시간 성공을 뜻하지 않는다. interval은1~60초다. 연속 실행은 terminal 또는 interruption에서 자동 종료한다. completed는exit0, failed/interrupted는exit1이다. 정상 collector 종료 뒤 같은 destination을 재사용하지 않고 이전 증거를 보존한다.

## bounded 보관과 안전 실패

고정 allowlist는 download status/result/worker-status/samples, runner status/result/test.log, guardian status/result/samples/ownership/guardian.log다. SQLite·미디어·환경·사용자 데이터·credentials·임의 파일은 복사하지 않는다. JSON입력64KiB, 각 보관파일1MiB, 전체8MiB, history512행 및1MiB 이중상한이다. source는64KiB chunk로 읽고 SHA를 계산하며 oversize/growth/aggregate reserve에서는 생략 이유를 남긴다. 이전 사본이 있으면 그 SHA/크기를 유지하므로 최신 source 전체 증거로 확대하지 않는다. source의 동시 쓰기 중 복사는 읽은 byte snapshot이며 cross-file transaction을 뜻하지 않는다.

owner UID, regular file, 절대 경로, parent/root/file symlink를 검사하며 directory FD 및 O_NOFOLLOW를 사용한다. source root identity 교체, destination root 교체, 임의 destination entry 또는 hardlink/symlink는 실패한다. 새 destination만 생성하고 기존 파일을 덮어쓰지 않는다. atomic publication은 새 소유 임시파일/O_EXCL→flush/fsync→rename이며 실패 시 자신이 만든 임시파일만 정리한다. 같은 사용자의 악의적인 filesystem 교체 전부를 완전히 격리하는 sandbox라고 주장하지 않는다.

## 상태와 완료 증거

현재 boot가 다르면 interrupted로 기록하고 새 boot의 같은 pathname 자료를 복사하지 않는다. 같은 boot에서 PID/start가 다르거나 zombie/부재이고 해당 그룹의 완전한 terminal 증거가 없으면 interrupted다. source root 부재도 interrupted다. stored running만으로 생존을 판정하지 않는다.

완료는 세 실제 result가 모두 completed이며 runner exit_code 정수0, download eight_hours_verified=true/finite elapsed≥28800, guardian eight_hours_elapsed=true/finite elapsed≥28800/descendants_reaped=true일 때만 기록한다. 명시 failed/stopped/interrupted 결과는 failed로 분류한다. 잘못된/불충분한 세 terminal은 failed다. guardian 그룹은 자체 완료 증거, download 그룹은 download+runner 완료 증거를 각각 검증한다. 먼저 끝난 그룹의 종료된 PID는 matches=false를 유지하며 expected_terminal_exit=true로 구분한다. 다른 그룹이 정상 live이면 watching을 유지하고 두 그룹이 모두 증명돼야 completed다. 한 그룹 완료가 다른 그룹의 예기치 않은 죽음을 면제하지 않는다. 일부 result만 있는 미완료 그룹은 여전히 모든 knownprocess가 live여야 watching을 유지한다. 불완전·짧은 단일 그룹 terminal도 즉시 failed다. root는 종료 후 artifact/SQLite/reap·binary/source 계약을 별도로 직접 통합 검증해야 한다. collector result는 실제 사이트나 GUI 검증을 대신하지 않는다.

## 실제 검증

`python3 -m unittest discover -s scripts -p test_soak_evidence_archive.py -v`: 23개 PASS, 최종1.15초. 합성 fixture와 현재 시험 프로세스 자기 PID 읽기만 사용했으며 subprocess/GUI/network/실제 soak를 실행하지 않았다. live snapshot/SHA/allowlist, complete/partial/failed/short/nonfinite terminal, boot change, PID reuse, missing root, oversized/corrupt JSON, oversized log, root/file/destination symlink, manifest mismatch, existing destination preservation, history/aggregate bounds 및 실제 허용파일 총량 초과 생략, 연결 후 root 소실, FIFO 비차단 거절, atomic 실패 소유임시정리를 확인했다. 통합검토에서 두 그룹 시작 시각 차이로 먼저 끝난 그룹 PID 부재를 중단으로 오인하는 경계를 발견해 수정했다. guardian-first/download-first 정상 종료·split완료 뒤 최종completed·다른 그룹 예기치 않은 죽음·단일 invalid terminal·unknown role 실패를 추가 검증했다. 실제8시간 결과·Windows/macOS·system suspend는 미검증이다.
