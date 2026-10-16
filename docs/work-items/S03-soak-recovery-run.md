# S03: WSL 중단 후 고정 바이너리 8시간 재시험

- 상태: in-progress
- 분류: verification
- 우선순위: P0
- 요구사항: R04, R08, R10, R11
- 담당/수정 범위: root; 새고유/tmp시험·고정binary사본·project evidence/launch, 본문서
- 선행 작업: S04-soak-evidence-archive의23실패경로 root독립PASS, current CPU71/default200/workspaceCPU325

## 사용자 결과

중단된8시간합성시험을새boot의독립ownedfixture로다시수행하고, 각명령·binary·boot·PIDstart·진행기록을프로젝트에보존한다.

## 계약과 범위

기존시험/원시기록복구또는덮어쓰기없다. coreDownloadRepository+SQLite와headlessguardian 두합성시험, realmedia/network/GUI범위아님. currenttestbinary/helper/CPUinternalguardian을Linux/tmp사본으로고정해향후Cargo빌드의파일교체와분리한다. SHA/크기/inode·boundedsourcehash를launch기록. root표준argv기존scripts만사용. 영구archive는명시owned3roots/boot10roles/start조건/파일1MiB·전체8MiB·history512한도. 정상scope종료시각차이를23회귀로확인했다. 실제8시간terminal/DBreopen/ownedcleanup전완료선언금지.

## 인수 조건

두프로세스readiness/현재boot-PIDstart·source/copySHA동일·archivewatching. 실제각28800초/성공exit·원DB최종상태/owneddescendantreap/archiveterminal·최종hash독립대조. WSL종료시다음세션이interrupted로구분한다.

## 검증과 handoff

collector23testsrootPASS1.087초. 새start는launch.json의exactargv/paths/ownership을기준으로감사한다. partial/final은합성64KiB/작은바이트이며userDocuments/영상아니다. 이전boot의약5시간59분기록을8시간으로합산하지않는다. guardianSIGKILL실패·실제media/sleep/diskfull은별도gate로남는다.
