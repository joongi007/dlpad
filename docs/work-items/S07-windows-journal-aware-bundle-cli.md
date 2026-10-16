# S07: 단계 journal을 함께 생성하는 개발 CLI

- 상태: done (개발 CLI15 및 실제 복사 제어 범위)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 API·통합·Windows 최종 검증·문서; metrics_recovery — /tmp CLI parser/report/test 초안; soak_recovery_audit — /tmp 독립 인수 검토; diagnostic_cache — 선행 증거 독립 검토.
- 선행 작업: S07-windows-journal-aware-fresh-bundle 실제 native 증거 인수. 선행 검증 중 프로젝트 Rust source를 고정하며 /tmp 초안만 병렬 작성한다.

## 사용자 결과

개발용 prepare-journal-user-bundle 명령이 새 bundle과 별도 journal을 실제 검증 단계 순서대로 생성하고 inspect-install-journal로 현재 일치 여부를 확인할 수 있다.

## 계약과 범위

AGENTS/README/PRODUCT/CONTEXT/R08/R11/ADR0005 계약을 유지한다. root가 crates/engine/src/app_install/journal_bundle.rs의 필요한 오류 보고 accessor를 조정한다. CLI 담당 초안은 crates/engine/src/bin/dlpad-local-install.rs의 parser/report 파일을 읽고 정확한 변경 파일 목록을 제출한다. scripts/windows_msvc_* collector 및 docs/development/windows-local-install.md, CONTEXT.md, 관련 work-item과 P0-status는 root만 통합한다. Cargo/lock/schema 변경은 현재 필요하지 않다. shell/launch/자동 resume/delete/정식 installer UI는 범위 밖이다.

## 인수 조건

기존 14개 명령 호환, 엄격한 필수·중복·잘못된 경로·옵션·기한 거절, actual public coordinator 사용, 성공 후 readonly current binding 확인. 취소·기한·잘못된 SHA·양쪽 기존 root에서 foreign 자료와 원 source 보존. 실패 후 mutation/cap 보유 및 미관측 개수는 정확히 보고한다. bounded JSON 한 줄 출력. 전체 fmt/clippy/test/release와 실제 Windows argv/exit/Job/source/artifact/retained ID·SHA 검증을 root가 수행한다. 역사 소유권·power-loss commit·삭제 권한·제품 installer 완료를 주장하지 않는다.

## 검증과 handoff

구현 초안과 선행 actual native 증거 인수 진행 중. 아직 CLI 구현·통합·실행 완료 아님.

선행 actual coordinator API native 검토를 인수하고 CLI 초안을 root 통합했다. 원 SHA 대조 후 정확한 단일 담당 파일만 변경했다. CLI15 미관측 단계 capcounts/mutation은 null+reportedfalse이며 기존14 report는 유지한다. root 공유 schema/API/Cargo 변경 없이 통합했고 이제 fmt 및 전체 Windows4검사를 실행한다.

root 실제 Windows fmt/clippy/test/release 통과: source586 정확 map의827/0/13. CPU5bcc9f…8,710,656B 불변, CLIc4cbc7…935,424B(+24,576B=24KiB). 신규의존성/feature/lock 변경·상시worker 없음. scripts/windows_msvc_journal_aware_cli_collect.py를 root 통합해 실제 CLI15 생성/별도 readonly 재검사/typed negative 시험을 진행 중이다. 앞선 fmt 적용은 exit0이나 source변경으로 guard passedfalse이며 전체 fmt --check가 이후 통과했다. docs 검사838PASS. Git 상태 fatal128 invalid를 재확인하고 .git 보존했다.

실제 CLI15 collector15명령 exit 인수 진행: 성공 생성→별도 journal·bundle readonly 호환 및 existing양root/wrongSHA/overlap/parser negative. scripts/windows_msvc_journal_aware_cli_control_collect.py와 windows_journal_aware_control_probe.cs 담당을 root로 추가하고 별도 독립 QA를 진행했다. 실제midcopy 시험 전 관측기의 live journal FILE_SHARE_READ 충돌을 발견해 전용 readonly reader만 READWRITE|DELETE 공유로 수정했다. 제품原 CreatedFile 잠금·기록을 바꾸지 않으며 source/CLI 원pin READ 공유는 유지한다. 원draft 실행전 결함이며 실제FAIL실행 증거를 만들어 쓰지 않는다.

실제 CLI15 native collector15명령(성공3/예상실패12)과 root C#7명령에서 journal/nativeID·canonical chain·실제4stage SHA·manifest 및 domain registration digest를 대조했다. 원 source·생성12files·sentinel/SHA보존 및 독립 raw15 검토 완료. root-review actual_midcopy_cli_control_verified=false는 다음 제어시험과 구별한다. outer current nativeID는 creator originalcap 증명이 아니다. 실제 취소/기한 수집기는 컴파일+cancel+timeout을 실행 중이다.

실제 copy-stage 취소/30s 기한을 compile+2case natural exit1/Job0/private console/input불변으로 인수했다. 원 소유 primary thread copy checkpoint 후 CTRL_BREAK 또는 schedulingpause→동일30sdeadlineTimedOut을 관측했다. root 추가 native9명령에서 intent 원root/file IDs·SHA/canonical manifest, source128MiB(currentCPU+zero padding) 및 실제 부분 copy prefix/foreign 보존을 비교했다. source/partial nativeID는 현재 readonly 관측이며 원 creatorcap역사증명이 아니다. 독립 raw3/sourcepins 검토를 보존했다. 증거 windows-msvc-journal-aware-cli-control-real-20261006-continue/root-review.json 및 independent-review.json. 실제 parser·관측기 source결함은 시험 전 수정했고 기준/guard 완화 없음. 전체4검사827/0/13·CLI15사례+native7명령·제어3명령+native9명령을 root와독립담당이 각scope 인수했다. 정식installer/upgrade·physicalstall·crashcommit·historicalownership·삭제authority는미완료. 다음은 S07-windows-native-development-bootstrap이며 최신 고정CPU GUIcost측정동안 /tmp만병렬구현한다.

현재 CLI15 고정 CPU5bcc9f…의18구간 모두PASS(최대CPU0.625703%, WS72,323,072B). root원counter/raw2/현재sourcepins/개별PNG3 검토와합성30,000파일 exact010203/정확namespace 및 SQLite3×10k/integrity/SHAprepost를인수했다. 독립18counter/raw2검토도보존했다. 증거 windows-msvc-journal-aware-cli-default-cost-real-20261006-continue/{root-counter-pixel-review.json,root-payload-db-review.json,independent-review.json}. 전체P0/physicalIME/DPI/reader/정식installer/quiet-host확정은아니다. 다음bootstrap변경전 sourcefreeze 검증 완료.
