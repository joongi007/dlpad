# S04: 기존 장시간 시험 읽기전용 재개 감사

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R03, S04
- 담당/수정 범위: diagnostic_cache; 이 신규 workitem, 신규 soak-resume-audit validation/evidence만
- 선행 작업: 기존 soak-restart-20261002 launch/PID/start evidence

## 사용자 결과

기존두8시간시험의현재실행identity/status/journal/archive를read-only확인하고terminal증거없으면진행중으로보고한다.

## 계약과 범위

기존soak/아카이브프로세스restart/stop/kill금지. host bootID·knownPID/startticks만읽고환경/credentials는읽지않는다. 기존archiveprefix수정금지,신규auditprefix만쓰기. owned synthetic SQLite mode=ro/immutable로쓰기·lock없이bounded조회하며활성DBsnapshot한계명시. source/Cargo/기존harness변경없음. session손실원인WSL재시작추정금지.

## 인수 조건

known9프로세스+archive identity/boot대조,live/archiveelapsed상태/terminal파일/아카이브상한·freshness/합성DBstate를기록. elapsed28800未충족·terminal미존재를성공으로바꾸지않음. 읽기전용hostaudit명령/타임스탬프/원시hash/미검증사항보존.

## 검증과 handoff

knownlaunch boot794988eb/PID5770·5886·5889·5869·5888·5896·5897·5899·5898/archive6202를읽기전용확인한다.

## 실제 결과

known9+archive PID/start/boot일치,live running약21190/21156초,SQLite Downloading/terminal없음,archivefresh/hash8개일치. 직접proc/exe FD hash가launch와다르고deleted proc/maps oldinode와FD metadata가다른중요한계발견. 원자료보존/root검토요청.8시간미완료/provenance검토필요.프로세스action없음/기존sourcearchive무변경.전용validation/evidence참조.
