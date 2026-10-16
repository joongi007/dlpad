# S04: Windows paired managed/cookie capture

- 상태: implemented; integration-validation-pending
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 신규 process/windows_paired_capture.rs와 본 문서만
- 선행 작업: root paired protocol/launcher/runtime/journal; metrics_recovery paired config

## 사용자 결과

인증 metadata/download의 한 helper·원 Job 완료를 관리 journal과 cookie journal 모두에 연결한다.

## 계약과 범위

Metadata→cookie0, Download→cookie1만 허용. 양 original Intent/receipt, 실제 helper creation과 durable Bound, pair digest를 검증한 뒤 양 unlock/Ack/Begin. Terminal/EOF/helper exit 일치 뒤 양 receipt resolve에 모두 성공해야 정상 결과. 부분 실패는 남은 pending/원 receipt 보존. raw metadata는 내부 검증용이며 사용자 로그 omission은 기존 download adapter 담당. 기존 파일/Cargo/module 변경 제외.

## 인수 조건

양 writer launch 시도 직후 drop, 한 helper만사용, lateTerminal Cancel guard 유지, typed unconfirmed, bounded drains. 별도 compile/native 통합 전 완료 주장 금지.

## 검증과 handoff

신규 source SHA256 `b739829d730929dd27752bc1375c3f53190d8ba4b50df181c4025a15302b4612` 동결. `run(spec,cancel,metadata,progress,journal,managedroot,index,stage,cookies)`는 명시적9-argument 계약으로 해당 함수만 clippy allowance를 갖는다. stage/metadata 조합을 먼저 Metadata0/Download1로 제한한다. managed original Intent와 cookie original Intent를 config로 전달하고 원 writer 두 개를 launch 시도 직후 drop한다. 실제 native Prepared 뒤 양 durable bind 성공, cookie original Intent clone에서 Bound 재구성 및 실제 bound identity 동등성 확인, actual managed record와 `PairedGuardianBinding::new`로 pair digest를 만든다. 양 release 성공 뒤에만 Ack/Begin을 보낸다. Terminal+EOF+helper exitcode 확인 후 managed reacquire/resolve와 cookie confirm_terminal이 모두 성공해야 정상 결과다. 하나의 resolve가 성공하고 다른 쪽이 실패해도 전체는 typed unconfirmed이며 남은 pending을 성공으로 지우지 않는다.

root가 paired protocol/launcher/entry/runtime/journal, metrics_recovery가 config를 담당한다. 기존 bounded capture/drain 및 terminal 이후 late Cancel을 보내지 않는 guard를 유지했다. raw 인증 metadata는 내부 반환값으로만 제공하며 기존 DownloadService 인증 omission을 제거하지 않았다. 원 writer/nonce/path/log는 새 공개 로그에 출력하지 않는다.

Cargo/rustfmt/native 실행·기존 source 변경 없음. 실제 public authenticated managed fixture로 success/meta failure/download failure/cancel, 두 journal의 실제 원 receipt/정리/원 source·sentinel·secret omission을 root가 검증해야 한다. 실제 helper kill/app crash/부분 receipt 완료의 실패 fixture도 후속 필요하다. **제품 public 연결/실제 Windows 검증 완료가 아니다.**

주 담당 Windows GNU CPU 전체 fmt·Clippy(-D warnings)·workspace test·release PASS: windows-paired-guardian-integrated-first. 최종 소스 SHA와 현재 파일을 독립 대조했다. 순수 binding/config/protocol 14개도 common cargo test로 통과했다. 실제 paired Job·두 receipt 성공/실패/취소 및 crash 인수는 아직 fixture 검증 대기다.

실제 paired 성공 인수: windows-paired-public-download-duration-retry에서 공개 Repository의 버전 확인 및 인증 metadata/download를 실행했다. 한 원 Job의 두 journal·receipt 확인, 인증 출력 생략, 원 cookie/hash·sentinel 보존, 원 소유 기록 정리·재사용을 통과했다. 주 담당이 job_json의 Downloaded/artifact, DB/WAL/SHM의 합성 SECRET 부재, 원 cookie SHA, cookie cache에 foreign sentinel만 남음, managed 9파일/pending 부재와 현재 소스/고정 binary SHA를 독립 대조했다. 실패·취소·부모 crash 및 startup 자동 회수는 아직 후속이다. 마지막 전체 Windows GNU CPU fmt/clippy/test/release도 windows-paired-native-fixture-workspace에서 통과했다.
