# S07: 개발 CLI의 외부 receipt 생성·대조·명시 제거

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — 정책·통합·공유 façade·최종 검증·문서; diagnostic_cache — /tmp CLI 초안 (crates/engine/src/bin/dlpad-local-install.rs 단독); metrics_recovery — /tmp 실제 synthetic CLI runner; soak_recovery_audit — /tmp 독립 실패 경로/인수 감사
- 선행 작업: 등록/외부 receipt/두경로 readonly 대조 구현 및 명시 cold 제거 scoped 인수. 현재 cold native receipt 검증을 마치기 전 제품 소스를 변경하지 않는다.

## 사용자 결과

개발 CLI에서 새 등록 묶음과 외부 receipt를 만들고 별도 실행에서 두 경로를 읽기 대조한다. `remove-registered`를 명시 호출했을 때만 정확한 묶음을 제거하며 외부 receipt·source·unknown 파일을 보존한다.

## 계약과 범위

추가 명령은 `prepare-register-receipt --manifest ABS --payload-dir ABS --target ABS --receipt-target ABS [--timeout-seconds 1..600]`, `inspect-pair --target ABS --receipt-target ABS`, `remove-registered --target ABS --receipt-target ABS`. 기존 네 명령은 유지한다. 명령별 flag/argv 상한을 적용한다. receipt target과 payload/manifest/등록 target의 양방향 overlap은 최초 mutation 전에 원 native final parent/leaf 비교로 거절한다. 기존 source identity/hash pre/post와 cancel/deadline 보호를 유지한다. stdout 4KiB 이내이며 raw path/SID/JSON receipt를 출력하지 않는다. 제거 단계별 accepted disposition·관측한 remainder cap 수·mutation 여부를 구별하고 CLI 종료 뒤 핸들이 살아있다는 주장을 하지 않는다. 실패에서 partial objects 자동 정리/재시작 채택을 하지 않는다. inspect와 startup은 DELETE factory를 호출하지 않는다. 신규 dependencies/schema/Cargo 변경 없음.

## 인수 조건

strict parser 성공/unknown·중복·불완전·잘못된 명령 flags 거절; Windows 전체 fmt/clippy/test/release; 실제 합성 CLI 자연 종료 이후 receipt 생성→pair 검사→명시 제거/외부 receipt 보존; 각각 유효한 다른 묶음 receipt 혼합·foreign·replacement·cancel/deadline·이미 있는 target·overlap에서 보존과 정확 mutation/caps 표시. 별도 raw/root/독립 증거를 대조한다. 일반 installer/active pointer/upgrade/서명/역사적 ownership 완료로 보고하지 않는다.

## 검증과 handoff

초안 병렬 작성 중. 제품 적용·실행 검증 전이다. 정상 Git 저장소 없음은 보존하며 branch/worktree 및 remote CI 실행으로 표현하지 않는다. clean OS/MSVC·active 설치와 upgrade 복구는 후속 P0다.

통합 소스 SHA979a727a…33ee의 Windows GNU 전체 fmt/clippy/test/release가 PASS이며 root/독립 raw 대조에서 app_install75·parser4회귀를 확인했다. CPU f3bae801…/7,917,568B 불변. CLI efcef0b0…/651,776B로 이전577,024B보다74,752B 증가하며 새 dependency/Cargo.lock 변경은 없다. 현재 실제 CLI 생성·두경로 대조·명시 제거/거절 인수를 실행 중이다. 엔진 수준 이전 cold 제거 성공을 새 CLI 인수에 소급하지 않는다.

실제28명령과 root/독립 검토 PASS: 새 생성→별도 readonly 검사→명시 제거 file5/rootgone/외부receipt보존, independently-valid 혼합receipt/foreign 거절0, 기존receipt beforeprepare AlreadyExists/target없음, 양방향 overlap/nativecase/verbatim alias 및 strict parser 실패의 원본 보존. 현재CLI 복사중 제어2case는 synthetic128MiB PE64/52n notice에서 Interrupted/TimedOut·phase prepare·원cap1/regreceiptcaps0·ready/registration/receipt없음, partial1,310,720B/6,356,992B 원prefix 일치, source/manifest/foreign/naturalJob/privateinput 보존 PASS. 기한은 원ownedprimarythread1100ms pause 실험이며 physicalstall이나 registration/receipt 단계 중단을 뜻하지 않는다. 제어helper rootID는 legacy64bitfileindex관측이며 full128 authority 검사로 확대하지 않는다.

별도 native 관측은 receipt3루트/파일과 남은 등록2루트/각5known파일, 총18개의 서로 다른 원128bit ID·정확u64volume·SHA·크기를 확인했다. unknown foreign는 등록authority로 조회하지 않고 원bytes/namespace를 보존한다. root/독립 원로그·실파일·원ID·현재source 대조 PASS. [전체](../validation/evidence/windows-app-receipt-cli-workspace-20261006-continue/root-review.json), [실제28명령](../validation/evidence/windows-app-receipt-cli-real-20261006-continue/root-review.json), [복사중제어](../validation/evidence/windows-app-receipt-cli-control-20261006-continue/root-review.json), [별도원ID](../validation/evidence/windows-app-receipt-cli-native-identities-20261006-continue/root-review.json). 원래 argv상대 evidence 경로를 넣은 1회는 argparse가 실행전거절했고 절대경로로 정정했다. 실제 실패 로그를 성공으로 바꾸지 않았다.

실행 명령: `python3 scripts/windows_current_workspace_probe.py /mnt/d/project/dlpad/docs/validation/evidence/windows-app-receipt-cli-workspace-20261006-continue --execute --seconds 900`, archived real collector `python3 /tmp/dlpad-receipt-cli-real-run.py OUTPUT --cpu-sha f3bae801… --cli-sha efcef0b0…`, control collector `python3 /tmp/receipt-cli-control-run.py --project PROJECT --output OUTPUT --cli CLI --cli-sha efcef0b0… --cpu CPU --cpu-sha f3bae801… --observer /tmp/receipt-cli-control.cs`, `python3 /tmp/receipt-cli-native-run.py`. 정확 argv·전체SHA·collector 원본은 각 evidence의 operations/inputs에 보존했다. 모두 실제 PASS_SCOPED이며 마지막 문서 검사 결과를 추가 기록한다. 다음 Windows현재패키지 notice scope/active 설치·upgrade 복구를 진행한다. CLI는 개발용이며 cleanOS/minOS/MSVC/서명/publicrelease/actualaccount 작업을 하지 않았다.
