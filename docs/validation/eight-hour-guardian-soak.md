# 8시간 Linux headless guardian synthetic soak

상태: **실제 장시간 시험 진행 중, 완료 아님**. 2026-10-01 Linux/WSL2에서 시작했다. 네트워크·GUI·사용자 Documents·실제 영상은 사용하지 않는다.

## 검증 범위와 실행

기존 release dlpad의 `--dlpad-internal-guardian` entry를 anonymous socket readiness와 함께 직접 실행한다. root, 중첩 middle→leaf, setsid detached Python 자손을 8시간 유지한다. 각 자손은 heartbeat를 갱신하고 root는 64 KiB 합성 partial을 5초마다 덮어쓴다. 지속 누적된 다운로드 파일이나 실제 영상이 아니다. 임시 fixture와 log/sample에 상한을 둔다.

```sh
python3 scripts/long_job_soak.py target/release/dlpad /tmp/dlpad-eight-hour-guardian-20261001 --seconds 28800 --interval 60
```

- 시작: **2026-10-01 12:49:48 UTC / 21:49:48 KST**.
- 8시간 목표 시각: 2026-10-01 20:49:48 UTC / **2026-10-02 05:49:48 KST** 이후. 실제 종료·결과를 확인해야 완료로 바꿀 수 있다.
- 소유 exec session: **57344**.
- owner PID 29575, guardian 29577, root 29579, middle 29580, leaf 29582, detached 29581. 종료에는 PID 숫자만 사용하지 않고 현재 owner가 유지한 pidfd/anonymous socket 또는 아래 owned stop 요청을 사용한다.
- release 앱 크기 7,958,304 bytes, SHA256 `61c369c8df261751c1e56f47f881d26b642405fc57c5b5610fd14d957d87e23e`.
- 실행 전 hash한 파일과 실제 `/proc/<guardian>/exe`의 dev/ino를 대조했다. 후속 빌드의 같은 경로 바이너리와 혼동하지 않는다.
- 첫 실제 sample: elapsed 0.083초, sample 1, partial 65,536 bytes, 네 역할 heartbeat count 1. owner RSS 17,656 KiB, guardian 4,620 KiB; synthetic Python 각 11,188~11,332 KiB. 시작 수치이며 장시간 결과가 아니다.

중간 실제 관측: elapsed **120.265초**, samples **3**, root/middle/detached heartbeat **24**, leaf **25**, partial **65,536 bytes**. guardian RSS **4,620 KiB**, 최근 분 CPU **0.1165%**, samples.jsonl **2,619 bytes**였다. `state=running`, `result.json` 미생성이다. 이것은 두 분까지의 진전 증거이며 8시간 성공이 아니다.

## 산출물과 안전한 중단

출력은 새 `/tmp/dlpad-eight-hour-guardian-20261001` 폴더다. `status.json`은 최신 상태, `samples.jsonl`은 분당 각 프로세스 RSS/CPU ticks/delta와 heartbeat, `ownership.json`은 이 fixture의 관측 identity, `guardian.log`는 합성 출력, `result.json`은 종료 시에만 생성한다. 중간 status나 elapsed 숫자만으로 완료를 주장하지 않는다.

필요한 경우 소유 시험만 멈추려면 다음 요청 파일을 만든다.

```sh
python3 -c "from pathlib import Path; Path('/tmp/dlpad-eight-hour-guardian-20261001/stop.request').write_text('Stop owned synthetic soak\n')"
```

하네스가 stop 요청을 확인하면 socket EOF로 guardian 정리를 실행하고 소유 fixture만 pidfd로 처리·정확 PID wait로 회수한다. 중단 결과는 `state=stopped`, exit 1이며 8시간 성공으로 기록하지 않는다. owner가 갑자기 죽어도 guardian socket EOF 경로가 동작한다. guardian 자체 SIGKILL에서는 직접 leader 이외의 자손이 생존할 수 있으므로 하네스 owner의 소유 pidfd cleanup을 보호 완료로 오인하지 않는다.

## 하네스 사전 검증

- 3초 fixture: 3 samples, elapsed 3.131초, `state=completed`, 소유 프로세스 전부 회수. `eight_hours_elapsed=false`.
- stop.request 짧은 시험: exit 1, stopped, 0.312초; 소유 프로세스 전부 사라짐.
- guardian만 SIGKILL 짧은 시험: exit 1, failed, 0.283초; harness가 소유 fixture만 정리하고 전부 사라짐을 확인.
- Python 3.10에서 hashlib.file_digest가 없어 최초 실행이 시작 전 실패했다. 64 KiB streaming SHA256으로 수정했고 실제 8시간 실행은 정상 시작했다.

samples는 최대 481행/512 KiB, logs 최대 256 KiB, partial 최대 64 KiB를 확인한다. 메모리에 stream 전체를 읽지 않는다. 새 dependency/Cargo/schema는 없으며 Python fixture 비용과 guardian 비용을 따로 기록한다. 추가 파일은 owned 임시 폴더 안에서만 생성하고, 기존 출력 디렉터리는 거절한다.

## 출시 게이트와 구별

이 시험은 **headless guardian의 wall-clock longevity/소유 프로세스 회수/시간별 RSS·CPU 관측**이다. 실제 DownloadRepository/yt-dlp/FFmpeg/DB/UI 다운로드 작업, 8시간 미디어·4 GiB 데이터·사이트 smoke·이어받기·URL 만료·네트워크 단절·sleep/resume를 검증하지 않는다. 해당 R04/S03 게이트와 guardian 자체 fatal 고장 보호는 여전히 미완료다.

실제 다운로드 adapter 장시간 시험에는 별도 Rust runner와 synthetic 도구 fixture를 main 담당자가 합의해 제공해야 한다. 그때 `DownloadOptions`/Attempt/partial/진행률/DB 상태를 실행·확인하고 guardian longevity 수치와 별도 결과로 기록한다. 기존 Rust 소스를 이 하네스 작업에서 변경하지 않았다.

## 2026-10-02 재개 시 증거 소실

현재 boot ID794988eb-539b-437e-8396-330738b565bb에서 이전 /tmpstatus와 소유 PID가 존재하지 않는다. 위 running 관측은 당시 중간 기록이며 현재 실행 상태가 아니다. session57344/76382의 8시간 완료 결과는 미검증이다. 기존 시험을 성공으로 처리하지 않는다. 신규 실행과 영구 증거는 S04-soak-restart 작업으로 구분한다.
