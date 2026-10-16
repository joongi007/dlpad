# Windows 개발용 로컬 앱 준비·등록·선택·감독 실행

`dlpad-local-install.exe`는 development_only 묶음을 명시한 로컬 경로에서 준비·검사·선택하거나, 선택한 앱을 감독하며 실행하는 개발 CLI다. publisher trust, 정식 installer, 자동 탐색, 업그레이드, 상주·영구 분리 실행을 제공하지 않는다. 기존 문서의 네·일곱 명령 검증은 각각 당시 snapshot의 범위이며 새 명령 전체 인수로 확대하지 않는다.

## 명령

| 명령 | 필수 옵션 | 동작 |
|---|---|---|
| prepare | --manifest, --payload-dir, --target | 새 development 묶음 복사 |
| inspect | --target | 준비 묶음 읽기 검사 |
| prepare-register | --manifest, --payload-dir, --target | 새 복사 후 등록 record/seal 결합 |
| inspect-registered | --target | 등록 묶음 읽기 검사 |
| prepare-register-receipt | --manifest, --payload-dir, --target, --receipt-target | 새 등록과 별도 외부 receipt 생성 |
| inspect-pair | --target, --receipt-target | 명시한 등록/receipt 일치 검사 |
| remove-registered | --target, --receipt-target | 명시적 제거 factory로 등록 묶음 제거, 외부 receipt 보존 |
| select-active | --target, --receipt-target, --active-target | 새 active-selection 기록 생성 |
| inspect-active | --target, --receipt-target, --active-target | 세 경로의 원 ID·해시·canonical 관계 검사 |
| launch-selected | --target, --receipt-target, --active-target | 검사한 정확한 선택 앱을 감독 실행 |
| run-selected | --target, --receipt-target, --active-target | startup 기한 뒤 자연 종료·취소까지 원 supervisor 유지 |
| prepare-user-bundle | --manifest, --source-dir, --bundle-root | 새 outer 안의 registered·receipt·active를 순차 생성 |
| inspect-user-bundle | --bundle-root | 명시 outer와 고정 세 자식의 읽기 전용 일관성 검사 |
| inspect-install-journal | --journal-root, --bundle-root | 명시 journal과 bundle의 현재 일관성 읽기 검사 |
| prepare-journal-user-bundle | --manifest, --source-dir, --bundle-root, --journal-root | 실제 검증 단계마다 별도 새 journal을 기록하며 bundle 생성 |
| remove-user-bundle | --bundle-root, --journal-root | 현재 완전한 개발 bundle을 명시 제거하고 외부 journal 보존 |

`run-selected` 이외 명령은 `--timeout-seconds` 1–600초, 기본300초를 받는다. `run-selected`는 `--startup-timeout-seconds` 1–600초, 기본300초만 받는다. 서로의 기한 옵션을 혼합하면 거절한다. 중복·unknown 옵션, command-specific 옵션 혼합, 상대·부모·제어문자·UNC 및 지원하지 않는 device 경로는 거절한다. 모든 경로는 명시한 로컬 절대 경로이며 존재/겹침·원 native ID와 namespace 검사는 해당 엔진 계약을 따른다. 읽기 검사와 실행은 기존 선택을 덮어쓰거나 새 선택을 자동 생성하지 않는다.


## prepare-user-bundle 사용

합성 개발 source와 아직 존재하지 않는 outer 경로를 명시한다. 이전 prepare 계열의 --payload-dir/--target 대신 --source-dir/--bundle-root를 받으며 혼합하면 거절한다.

```text
dlpad-local-install.exe prepare-user-bundle --manifest C:\Temp\dlpad-fixture\manifest.json --source-dir C:\Temp\dlpad-fixture\source --bundle-root C:\Temp\dlpad-fixture\bundle --timeout-seconds 300
```

source 전체 SHA 검증 이후 새 outer와 고정 registered/receipt/active 자식을 원 HANDLE-relative로 생성한다. 등록·receipt를 대조한 뒤 active를 마지막에 게시한다. 기존 outer는 채택하거나 교체하지 않는다. 성공은 PREPARED_USER_BUNDLE_DEVELOPMENT_ONLY이며 자동 실행하지 않는다. 이후 inspect-active/run-selected의 세 경로로 bundle/registered, bundle/receipt, bundle/active를 명시한다.

중간 실패에는 bundle_stage·mutation 가능성·종료 전 원cap 보유 여부를 보고한다. generic preflight 거절에는 coordinator 전용 필드가 없을 수 있다. 실제 cap 총수는 보고하지 않으며 Drop/실패에서 부분 파일을 자동 삭제하지 않는다. durable journal·upgrade·rollback·기본 설치 위치·자동 탐색은 제공하지 않는다. 현재 source579 Windows 전체4검사·774/0/13을 root/독립 검토했으며 새 CLI14명령·GUI3개/재시작·원NativeID10 및 별도 복사 중 취소/기한3명령·2사례를 root/독립 검토했다. 최초 수집기 key-case FAIL은 보존하며 이는 해당 source579 정확 hash map의 범위다.

## prepare-journal-user-bundle 사용

서로 겹치지 않는 새 bundle·journal 두 경로를 명시한다. 실제 Prepared/Registered/Receipt/Active 검증 직후 각각 기록하며 완료 뒤 이력을 추가하는 방식은 사용하지 않는다.

```text
dlpad-local-install.exe prepare-journal-user-bundle --manifest C:\Temp\dlpad-fixture\manifest.json --source-dir C:\Temp\dlpad-fixture\source --bundle-root C:\Temp\dlpad-fixture\bundle --journal-root C:\Temp\dlpad-fixture\journal --timeout-seconds 300
```

성공은 `PREPARED_JOURNAL_USER_BUNDLE_DEVELOPMENT_ONLY`다. 별도 `inspect-install-journal` 및 `inspect-user-bundle`로 현재 일치를 재검사한다. 기존 대상·잘못된 SHA·경로 겹침·잘못된 옵션에서는 기존 자료를 보존한다. 복사 중 취소·기한 실패에서도 journal과 부분 bundle을 자동 삭제하지 않는다. aggregate mutation과 종료 직전 양쪽 capability 보유를 보고하고, 미관측 단계별 capability 개수·mutation·단계는 null/unreported로 표시한다. journal은 역사적 소유권·power-loss commit·자동 복구·삭제 권한이 아니다.

Windows 전체4검사827/0/13·실제 CLI15사례(성공3/예상실패12)·추가 native7명령을 root/독립 검토했다. 실제 복사 중 취소·30초 기한 시험(3명령/2case)과 추가 native9명령에서도 부분 bundle·intent 원 ID/SHA·source·foreign 보존을 인수했다. 기한 시험의 thread pause는 scheduling 검증이며 물리 I/O stall 증거는 아니다. CPU5bcc9f…/8,710,656B 불변, CLIc4cbc7…/935,424B(24KiB 증가), 신규 의존성/feature/lock 변경·상시worker 없음. [작업 범위](../work-items/S07-windows-journal-aware-bundle-cli.md).

## inspect-user-bundle 사용

명시한 outer 하나를 읽기 전용으로 검사한다. 고정 `registered`, `receipt`, `active` 세 자식과 원 root·파일의 관계를 검증하며, 누락·외부 자식·불일치·취소·기한 오류에서는 원 파일을 보존한다.

```text
dlpad-local-install.exe inspect-user-bundle --bundle-root C:\Temp\dlpad-fixture\bundle --timeout-seconds 300
```

성공 결과는 `INSPECTED_USER_BUNDLE_DEVELOPMENT_ONLY`다. 원 readonly outer와 selection/pair 검사 capability를 검증 동안 유지한다. 관측하지 않은 capability 수는 `null`, `cap_count_reported=false`로 표시한다. 이 명령의 실제 Windows 전체4검사813/0/13·17명령/7행·원11nativeID·typed실패4/namespace실패2를root/독립인수했으며, 정식 installer·repair·실행·삭제·publisher trust의 완료 결과가 아니다. [작업 범위](../work-items/S07-windows-user-bundle-readonly-reopen.md).

## inspect-install-journal 사용

새 journal API를 연결한 14번째 명령이다. source583 Windows 전체4검사819/0/13, 실제 CLI12명령·원24파일·추가 native32ID 대조를 root/독립 인수했다. 서로 명시한 두 경로의 원 readonly capability로 journal 연속 단계와 실제 bundle manifest/active record의 현재 일관성을 검사한다.

```text
dlpad-local-install.exe inspect-install-journal --journal-root C:\Temp\dlpad-fixture\journal --bundle-root C:\Temp\dlpad-fixture\bundle --timeout-seconds 300
```

성공은 `INSPECTED_INSTALL_JOURNAL_DEVELOPMENT_ONLY`와 `current_consistency_verified=true`다. journal fingerprint만으로 과거 설치 실행·crash commit·소유권·삭제/자동 복구 권한을 주장하지 않는다. 경로와 전체 기록은 JSON에 포함하지 않으며 미관측 cap 수는 null로 표시한다. 미완료·unknown·torn·불일치 기록을 보존한다. 정식 installer 연결·UI·자동 복구는 후속 작업이다.

## launch-selected 사용

아래는 소유 합성 개발 fixture의 예시이며 사용자 데이터에서 실행하라는 지시가 아니다. 세 경로를 모두 명시해야 한다.

```text
dlpad-local-install.exe launch-selected --target C:\Temp\dlpad-fixture\registered --receipt-target C:\Temp\dlpad-fixture\receipt --active-target C:\Temp\dlpad-fixture\active --timeout-seconds 300
```

CLI는 등록/외부 receipt/active-selection을 원 readonly capabilities로 검사하고 선택한 `dlpad.exe` 이미지의 원 identity와 bytes를 확인한다. 원 suspended child를 소유 Job에 결합한 뒤 재개하고 종료까지 감독한다. argv override나 임의 inherited HANDLE를 GUI 앱에 전달하지 않는다. 경로를 알아냈다는 사실이나 stdout receipt만으로 소유권을 새로 얻는 것은 아니다.

CLI를 실행한 console에서 CTRL_C 또는 CTRL_BREAK로 취소하거나 지정한 기한이 끝나면 감독 실행을 멈추고 소유 child Job의 종료를 확인한다. supervisor 종료도 소유 Job의 kill-on-close 보호 대상이므로 CLI를 닫은 뒤 앱을 계속 사용하는 영구 detach 방식이 아니다. 종료 확인 실패는 성공으로 보고하지 않는다. 외부 선택 도구, 사용자 파일, 등록/receipt/active-selection 기록을 취소 때문에 자동 삭제하지 않는다.

기한은 명령 시작부터 검사·해시·실행을 포함하는 동일 deadline이다. 협력적 I/O 체크포인트와 유한 native 종료 정리 예산을 사용하며 막힌 모든 OS 호출이 정확히 지정 초에 중단된다는 보장은 아니다. 기본300초가 끝나면 정상 사용 중인 GUI도 감독 취소될 수 있으므로 실행 목적에 맞는1–600초를 명시한다.

## 실행 receipt와 실패

stdout은 ≤4KiB JSON 한 줄이며 원 경로·SID·전체 record를 싣지 않는다. 정상 child exit0 및 Job terminal 확인 시 CLI exit0/status `LAUNCHED_SELECTED_DEVELOPMENT_ONLY`, phase `launch-wait`, reason `Natural`이다. 자연 종료라도 child nonzero면 성공이 아니다. 취소는 status FAIL/reason `Cancelled`/kind `Interrupted`, 기한은 FAIL/`TimedOut`/`TimedOut`이고 CLI exit1이다. launch 전 검사 실패와 child 생성/재개 이후 실패를 구분한다.

`child_created`, `resume_attempted`, `may_have_executed`, `job_terminal_confirmed`와 실패의 `terminal_unconfirmed`를 각각 읽는다. 실행을 시도한 뒤 실패했다고 child가 실행되지 않았다고 추정하면 안 된다. `original_owner_retained_before_drop` 및 `selection_capabilities_held_before_drop`는 receipt 생성 당시 관측이며 프로세스 종료 뒤 HANDLE가 남는다는 뜻이 아니다. stdout은 지속적 trust/삭제 권한이나 정식 설치 완료를 증명하지 않는다.

## 비용과 검증 범위

선택 실행 전 등록 묶음의 full hashing/ID/ACL/namespace 검증은 파일 크기에 비례하는 I/O다. bounded chunks를 사용하고 UI thread에 수행하는 작업이 아니다. 현재 전체 빌드 proof(source576)에서 CPU54c33d95…/7,917,568B는 불변이고 기본 열 명령 CLI5ae864b8…/754,688B는 기존9명령 대비31,232B 증가했다. 의존성/Cargo.lock 변경은 없다. 전체 fmt/clippy/test/release 및758passed/13ignored를 주 담당과 독립 담당이 확인했다. actual GUI와 설치 공급 완료는 별도로 판정한다.

현재 기본 launch-selected의 private 실제 네GUI(first System→Light, restart Light, 원 CLI process-group CTRL_BREAK,15초 기한)와 Natural17/거절9/native3query 인수는 완료했다. 최종 receipt·자연 Job0·원파일34/ID10·SQLite/PNG를 주 담당과 독립 담당이 확인했다. 기존9명령은 각 과거 snapshot 인수이며 최신10명령 모두를 동일 binary에서 재시험했다는 뜻은 아니다. Unicode 합성 입력/OS IMM fixture/물리 IME, GNU/MSVC, 개발 묶음/정식 배포는 별도 증거 범위다. 서명·clean-machine·업그레이드·release-ready/전체 Windows P0 완료는 여전히 이 문서의 완료 주장이 아니다.

---

다음은 이전 snapshot의 명령 및 인수 기록이다. 위의 열 명령 안내를 현재 사용법으로 따른다.

# Windows 개발용 로컬 앱 준비·등록·재검증

개발용 `dlpad-local-install.exe`는 원 로컬 빌드를 새 폴더에 복사하고 선택적으로 등록 record/seal을 결합한다. 별도 프로세스가 이를 읽기 검증한다. 앱 자동 실행·active installation·정식 설치 프로그램·업그레이드·삭제 권한은 제공하지 않는다.

## 네 명령

정확한 workspace toolchain/lock으로 `cargo build --workspace --release --locked --features cpu`를 사용한다. 다음 경로는 합성 개발 예시이며 사용자 데이터에서 실행하는 지시가 아니다.

```text
dlpad-local-install.exe prepare --manifest C:\Temp\dlpad-fixture\manifest.json --payload-dir C:\Temp\dlpad-fixture\source --target C:\Temp\dlpad-fixture\prepared --timeout-seconds 300
dlpad-local-install.exe inspect --target C:\Temp\dlpad-fixture\prepared --timeout-seconds 300
dlpad-local-install.exe prepare-register --manifest C:\Temp\dlpad-fixture\manifest.json --payload-dir C:\Temp\dlpad-fixture\source --target C:\Temp\dlpad-fixture\registered --timeout-seconds 300
dlpad-local-install.exe inspect-registered --target C:\Temp\dlpad-fixture\registered --timeout-seconds 300
```

prepare-register는 새 target에서만 원 prepare 후 fresh registration을 수행한다. 기존 prepared 폴더를 경로만으로 등록하는 명령은 없다. prepare/inspect는 기존 3파일 예시를 유지하며 등록된 5파일 폴더를 legacy inspect에 주면 extra namespace로 거절한다. inspect-registered는 원 readonly HANDLE에서 ready·record·seal·payload를 검증하고 Drop 때 삭제하지 않는다.

입력은 일반 로컬 절대 경로, source와 target 비겹침, strict flag pairs를 요구한다. manifest/source/target의 sibling 배치는 허용한다. 상대·부모·제어문자·UNC/device·중복/unknown 옵션은 거절한다. timeout 1–600초, 기본300초는 협력적 체크포인트 기한이며 막힌 OS 호출의 강제 중단 예산이 아니다.

## 파일·등록 계약

Manifest는 64KiB 이하 schema1, 정확 version/Windows target, development_only=true, 실제 dlpad.exe와 허용 notice의 size/SHA를 제공한다. 파일당128MiB·합256MiB·64항목 상한이다. source ACL은 수정하지 않고 원 source 읽기 HANDLE를 유지하며 write/delete와 충돌시 거절한다.

등록은 원 root에서 create-only `install-registration.json`(64KiB), `install-registration-seal.json`(4KiB)을 추가한다. 원 root/ready/record/seal/payload IDs, canonical prepared SHA, domain-separated canonical registration SHA와 원 random nonce를 결합한다. 모든 원 bytes/IDs/ACL/정확 namespace를 다시 검사한다. seal은 서명이나 비밀 MAC이 아니며 동일 계정 공격 방어·역사적 삭제 권한을 증명하지 않는다. malformed/foreign/partial/Busy에서는 원 자료를 보존한다.

## JSON receipt와 실패

stdout은 ≤4KiB JSON 한 줄이다. 성공 exit0, 실패 exit1/status FAIL과 fixed kind다. 네 성공 status는 PREPARED_DEVELOPMENT_ONLY, INSPECTED_DEVELOPMENT_ONLY, REGISTERED_DEVELOPMENT_ONLY, INSPECTED_REGISTERED_DEVELOPMENT_ONLY다.

공통 schema1/operation/development_only=true, trusted_provenance/publisher_trust/installer_complete/upgrade_verified/deletion_authority/active_installation=false를 명시한다. path/SID/record·source 내용은 stdout에 넣지 않는다. 등록 두 성공만 registration_verified=true다.

prepare-register 성공에는 전체 mutation=true, registration_mutation_may_have_occurred=true, new_registration_file_caps=2다. inspect-registered는 mutation=false/newcaps0다. prepare 뒤 등록 실패는 전체 mutation=true이며 원 prepared cap 수와 새 confirmed 등록 cap 수를 구분한다. retained_file_caps는 반환 시 확보한 원 capability 수이고 process 종료 뒤 살아 있는 HANDLE 수 또는 모든 생성 객체 수가 아니다. 첫 native 생성 시도 이후 mutation flag는 보수적이다.

CTRL_C/CTRL_BREAK와 deadline은 추가 작업을 멈추지만 생성한 부분 파일을 자동 삭제하지 않는다. 원 partial/foreign 자료는 보존한다. fresh discard는 별도 engine API이며 이 CLI에는 제거 명령이 없다.

## 비용·검증 범위

CLI는 577,024B로 이전524,800B보다52,224B 증가했다. CPU GUI는7,917,568B로 동일하며 신규 dependency/lock 변경이 없다. 등록 canonical JSON은 각각64KiB/4KiB 상한이다. 추가 등록·최종 source pin 검사에는 bounded streaming full hash가 포함되어 파일 크기에 비례한 추가 I/O가 발생한다. 64KiB chunk는 전체 파일 적재를 뜻하지 않으며 상시 GUI worker를 추가하지 않는다. 실행시간 개선이나 물리 디스크 실패 내성을 측정하지 않았다.

Windows GNU 현재 전체 fmt/Clippy/test/release, engine 설치53회귀와 CLI parser3회귀를 통과했다. 실제 fixture16명령의 정상 등록·새 프로세스 읽기 검사·legacy 호환·정확 오류 종류·원 source/foreign 보존·canonical seal을 root와 독립 담당이 확인했다. 별도 FileIdInfo 관측으로 root와5파일의 원128bit IDs/size/SHA도 대조했다. [전체 검사](../validation/evidence/windows-app-registered-cli-workspace-20261006-continue/root-review.json), [실제 명령](../validation/evidence/windows-app-registered-cli-real-20261006-continue/root-review.json), [원 ID 대조](../validation/evidence/windows-app-registered-cli-native-identities-20261006-continue/root-review.json).

현재 CLI의 legacy prepare 중간 취소·controlled expiry 재검증도 통과했다. 원128MiB/source·notice·foreign과 partial prefix를 보존했고 ready marker는 생성되지 않았다. 원owned primary thread1100ms 정지는 실제 기한 오류를 만드는 합성 시험이다. [제어 결과](../validation/evidence/windows-app-registered-cli-control-20261006-continue/root-review.json). 이전 snapshot의 console 시험을 등록 명령의 중간 취소 인수로 확대하지 않는다. [작업 기록](../work-items/S07-windows-registered-local-cli.md).

MSVC/최소 OS/clean machine/서명·정식 설치·cold uninstall·업데이트는 후속 게이트다. 로컬 개발 등록 완료는 전체 Windows P0 완료를 뜻하지 않는다.

## 원 등록 receipt 엔진 API

Windows engine의 `create_registration_receipt(&FreshRegistration, target, cancel, deadline)`는 이번 실행의 원 등록 capability에서 별도 새 private 폴더에 `registration-receipt.json` 하나를 기록한다. 총64KiB·nested admission64KiB-2048 상한이며 원root/file IDs와등록record/seal을결합한다. 기존경로·등록폴더내부/겹침은거절하고실패/Drop에서삭제하지않는다. 공개API경로는UTF16 32767units/NUL검사를생성전수행한다.

`inspect_registration_receipt(path, cancel, deadline)`는 새프로세스의원읽기cap에서canonical bytes·privateACL·정확1namespace·원root/file IDs를재검증한다. receipt는등록target경로를포함하지않아그앱의현재존재/삭제authority/역사적소유권을증명하지않는다. 두 API는현재CLI명령이아니며상시GUIworker를추가하지않는다.

생성source563 전체4검사/61회귀·실제5case/원native2IDs, 읽기source565 전체4검사/65회귀·원receipt의별도process/4typed실패를검증했다. JSON2,305B는실제fixture크기이고64KiB전체boundaryfixture의증명이아니다. 기존payload 전체검증은파일크기에비례하는추가I/O이며64KiB stream·동일cancel/deadline을유지한다. [생성범위](../work-items/S07-windows-registration-receipt.md), [재시작검사](../work-items/S07-windows-receipt-reopen.md).

`inspect_registered_with_receipt(registered_path, receipt_path, cancel, deadline)`는명시한두경로의원readonlycaps를함께유지하고fullcanonicalrecord/seal을대조한다. source567 전체4검사/설치68회귀·원묶음별도process/4typedfail를통과했다. 다른유효등록의receipt혼합은별도nativeunit회귀로거절했다. GUI7,917,568B/새dependency0이며payload전체검증I/O와과거GUI측정은구분한다. [범위·원명령·증거](../work-items/S07-windows-registration-receipt-pair.md).

현재 engine 명시 제거 인수는 source570 전체4검사/75회귀·실제4묶음·7행 및 외부receipt4 nativeID 대조 PASS다. `authorize_explicit_registered_removal(pair, cancel, deadline)`는 명시 호출에서만 사용하고 `remove` 실패의 opaque `retry`는 같은 프로세스의 원caps에 한정한다. 이 engine 인수 시점의 네 명령 CLI는 receipt 생성/제거를 노출하지 않았다. 후속 일곱 명령 CLI 통합과 검증은 아래 별도 범위다. [진행 CLI 연결](../work-items/S07-windows-registration-receipt-local-cli.md), [제거 범위](../work-items/S07-windows-registered-cold-removal.md). 전체 installer/active/upgrade/clean OS/서명 인수는 미완료다.

## 외부 receipt가 있는 개발 묶음

현재 CLI에 `prepare-register-receipt`, `inspect-pair`, `remove-registered`를 추가했다. 전체 Windows GNU fmt/clippy/test/release와 parser4회귀는 통과했고 실제28명령의 생성/대조/거절/제거는 root 검토를 통과했다. 현재 복사중 취소·기한2case 및 별도원native18ID의 root/독립 대조도 PASS이며 정식 installer 완료 상태가 아니다.

```powershell
# 모든 경로는 소유 합성 TEMP fixture를 가리키는 절대 경로로 지정한다.
# receipt는 payload, manifest, 등록 대상과 겹치지 않는 새 외부 폴더여야 한다.
dlpad-local-install.exe prepare-register-receipt --manifest $manifest --payload-dir $payload --target $registered --receipt-target $receipt --timeout-seconds 60
dlpad-local-install.exe inspect-pair --target $registered --receipt-target $receipt --timeout-seconds 60
# 위 정확한 묶음을 명시적으로 제거한다. 외부 receipt는 보존한다.
dlpad-local-install.exe remove-registered --target $registered --receipt-target $receipt --timeout-seconds 60
```

생성 성공은 `REGISTERED_WITH_RECEIPT_DEVELOPMENT_ONLY`, 두 경로 검사 성공은 `INSPECTED_PAIR_DEVELOPMENT_ONLY`, 명시 제거 성공은 `REMOVED_REGISTERED_DEVELOPMENT_ONLY`다. stdout은4KiB 이내이며 경로/SID/전체 receipt를 담지 않는다. 생성 시 receipt가 이미 있으면 등록 복사 전에 `AlreadyExists`로 거절한다. source/manifest/등록 target과의 양방향 겹침도 mutation 전에 거절한다. 원 native parent/finalpath를 사용하므로 case/verbatim alias로 우회하지 않는다. 없는 ancestor 등 다른 경로 오류는 별도 ErrorKind일 수 있다.

실패 JSON의 `phase`, `mutation_may_have_occurred`, `receipt_mutation_may_have_occurred`, `new_registration_file_caps`, `new_receipt_file_caps`, `accepted_file_dispositions`, `retained_file_caps`를 구분한다. `cap_counts_are_pre_drop_observations=true`이고 `capabilities_retained_after_exit=false`다. 실패 partial을 자동 정리하거나 새 프로세스가 path로 remaining capability를 다시 채택하지 않는다. 동일 caps 재시도는 engine API의 같은 실행 안에서만 가능하며 CLI 재실행 보장이 아니다. accepted disposition은 물리적 secure erasure가 아니다. 검사만으로 삭제를 실행하지 않으며 `explicit_removal_requested`는 명시 제거 명령에만 true다. 공통 `deletion_authority=false`는 출력 JSON이 지속적/역사적 삭제 권한을 증명하지 않는다는 의미다.


## 명시 설치 선택의 개발 기반

engine `select_registered_pair(pair, active_target, cancel, deadline)`는 이미 검증된 등록 묶음과 외부 receipt를 새 private 폴더의 canonical `active-selection.json`에 결합한다. `inspect_active_selection(active_path, registered_path, receipt_path, cancel, deadline)`는 세 경로를 모두 명시해 원 ID·SHA·정확 namespace를 readonly 대조한다. 저장된 경로에서 자동 실행하거나 기존 선택을 교체하지 않는다. postwrite 실패 뒤 기록을 읽을 수 있어도 이전 작업 성공이나 sync 완료의 증명은 아니다.

Windows전체4검사/설치87회귀와 실제6case·별도process검사·정상묶음원nativeID10은 인수했다. CLI `select-active`/`inspect-active` 연결은 현재 검증 중이며 공개 설치나 업그레이드 완료가 아니다. [작업과 원증거](../work-items/S07-windows-active-install-coordinator.md).

## run-selected 사용과 receipt

```text
dlpad-local-install.exe run-selected --target ABS_REGISTERED --receipt-target ABS_RECEIPT --active-target ABS_ACTIVE --startup-timeout-seconds 300
```

명시한 세 로컬 절대 경로만 사용하며 자동 discovery·argv override·환경 override·shell·파일 삭제가 없다. 원 supervisor가 앱 종료까지 살아 있어야 하며 콘솔 종료 시 원 child도 끝날 수 있다. 일반 shortcut·영구 분리 launcher가 아니다.

Natural0와 원 Job 종료 확인이 모두 있어야 `RAN_SELECTED_DEVELOPMENT_ONLY`/exit0다. Natural nonzero는 FAIL/Other와 원 exit_code/reasonNatural, 취소는 FAIL/Interrupted/reasonCancelled를 보존한다. `terminal_unconfirmed`는 종료 확인 실패다. `runtime_deadline_seconds:null`, `runtime_until_cancel_or_exit:true`, `startup_timeout_seconds`를 구분한다. 현재 공유 구현에서 parser/general 실패는 preflight, 검사 실패는 launch-inspect, 시작 실패는 launch-selected이며 실제 대기 receipt부터 run-wait다. operation은 run-selected로 유지한다.

원 총 capability 개수는 추정하지 않고 `retained_file_caps:null`, `retained_file_cap_count_reported:false`다. `selection_capabilities_held_before_drop`와 `original_owner_retained_before_drop`는 pre-drop 관측이며 `capabilities_retained_after_exit:false`는 그대로다. stdout4KiB와 공통development_onlytrue/trust·publisher·installer·upgrade·삭제권한 false를 유지한다. nominal100ms 대기는 physical100ms 취소 SLA가 아니다. 원 HANDLE·File pins와 supervisor 수명 비용이 있고 실제 GUI CPU/장시간 비용은 미측정이다.

새 MSVC target 전체 fmt/clippy/test/release는 완료했다. 테스트 집계760 PASS/0 FAIL/13 ignored이며 source576은 이전 같은 파일 수의 snapshot과 다른 정확 hash map이다. CPU c9f09dc0…8,709,632B, CLI ab8a6cde…795,136B다. 이전 MSVC CLI786,944B 대비8,192B 증가했고 CPU 크기는 같다. 신규dependency/lock 변경은 없으며 같은 CPU 크기가 동일 binary SHA나 과거GUI 비용 검증을 뜻하지 않는다. 실제 시작2초 기한을 넘긴 실행4초 이상 유지·자연 종료·취소·밝은 테마 재시작을 root/독립 검토했다. 새 CLI 실제17명령/14사례·원nativeID10·원파일 보존도 통과했다. 기존 bounded 실행의 새 바이너리 회귀는 실제15명령/13사례·GUI4까지 root/독립 PASS다. 부모 종료 후 자식만 살아 있는 분기는 새 source에서 별도 검증 중이다. [현재 전체검사](../validation/evidence/windows-msvc-run-selected-corrected-workspace-20261006-continue/result.json). 정식 설치·upgrade·서명·publisher/legal/cleanOS와 P0 전체 완료는 미주장이고 ADR0005는 Proposed를 유지한다.


## remove-user-bundle 사용

명시한 완전한 개발 bundle과 외부 journal의 현재 일치를 검사한 뒤 활성 선택 표시, payload·등록 기록·receipt, 빈 하위 폴더·outer 순으로 제거한다. 기존 등록만 제거하는 `remove-registered`와 대상 범위가 다르다. 새 경로를 추측하거나 journal에서 자동 삭제 권한을 복원하지 않는다.

```text
dlpad-local-install.exe remove-user-bundle --bundle-root C:\Temp\dlpad-fixture\bundle --journal-root C:\Temp\dlpad-fixture\journal --timeout-seconds 300
```

완료는 `REMOVED_USER_BUNDLE_DEVELOPMENT_ONLY`와 현재 outer 부재 관측을 함께 보고한다. 원본·manifest·외부 journal·data·영상·도구는 보존한다. 취소·기한·오류에는 이미 수락한 disposition과 mutation 가능성을 보고하고 남은 원 자료를 자동 정리하지 않는다. 프로세스 종료 시 추가 disposition을 요청하지 않으며, 이전에 수락한 delete-pending 작업은 핸들이 닫히며 완료될 수 있다. 실패 stdout의 cap 수는 반환 전 논리 capability 관측이며 OS HANDLE 총수나 종료 뒤 복구 가능한 권한이 아니다. CLI는 partial 상태를 자동으로 이어 제거하지 않는다. [작업과 실제 검증 상태](../work-items/S07-windows-explicit-user-bundle-removal-cli.md).
