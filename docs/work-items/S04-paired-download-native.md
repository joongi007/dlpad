# S04: Windows paired 인증 다운로드 실제 fixture

- 상태: implemented; native-validation-pending
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 신규 examples/windows_paired_download_fixture.rs와 본 문서만
- 선행 작업: root public paired caller/helper, managed selection, cookie private cache

## 사용자 결과

공개 Repository에서 합성 쿠키+managed 실행으로 version/paired metadata/download와 Downloaded artifact, 양 안전 cleanup을 확인한다.

## 계약과 범위

현재 root source freeze 중이므로 Rust는 `/tmp/dlpad-windows-paired-download-fixture.rs`에 staging하고 게시 GO 전 프로젝트에 추가하지 않는다. 모든 TEMP·도구·Netscape cookie는 합성 소유 fixture뿐이며 실제 계정/영상/네트워크 없음. 실패는 owned root/marker/pending을 보존한다. 성공만 원 cookie cleanup/managed finish/reuse를 검증한다. 다른 파일/Cargo 변경 제외.

## 인수 조건

version argv에는 cookie 없음; metadata/download 같은 privatecopy literalbytes; actual JobDownloaded/artifact, secret output omission/metadata 자유 field 제외/DB path·secret 부재. 실제 원 receipts 확인 후 public cleanup, 관리9files 제거/reuse, source/sentinel/tool bytes 보존. success 우선이며 실패/취소/app kill은 후속 미검증.

## 검증과 handoff

root collector19285 실제 terminal0/게시 GO 후 새 example을 create-only 게시했다. SHA256 `eac8e38d54bd93e7707766264089c62db4a91e3a576c3d00346c3306a1790e93`. 공개 engine에는 SessionCookieGuard 타입이 없으므로 실제 `SessionCookieLease::cleanup`을 정상 실제 결과 후 사용하고 실행 실패 시 `defer_cleanup`한다. 앱 전용 guard를 가짜 public API로 사용하지 않는다. native 실행/Cargo는 root 담당.

복사된 self fake yt-dlp는 TEMP direct-prefix/CAP/managed exactpath/sentinel을 확인한다. version argv 정확3개/no cookie, metadata/download는 같은 원 privatecopy 합성 Netscape bytes를 확인하고 secret stdout/stderr를 발생시킨다. 공개 DownloadRepository 실행 뒤 omission 고정문구/자유 metadata title 제외/합성 duration/JobDownloaded/artifact 원 bytes, SQLite DB/WAL/SHM 각8MiB 범위의 source/privatepath/secret 부재를 검사한다. actual managed3 stages Resolved/receipt 각256 및 cookie2 receipt 각256을 확인하고 원 lease cleanup→managed finish→9 files 제거/reuse, source SHA/CAP/sentinel/tool bytes 보존을 요구한다. 성공 후 fresh JSON manifest를 쓰고 owned fixture root를 보존하여 root가 DB/artifact 독립 검토할 수 있다.

success 세로 slice만 작성했으며 실제 compile/native 결과는 아직 없다. metadata/download 실패·취소·helper/app kill·부분 receipt는 후속 미검증이다. 실제 계정·네트워크·영상/Cargo dependency/기존 source 변경 없음.

주 담당 검토에서 parent 원 journal이 byte-range lock을 보유하는 동안 다른 HANDLE로 JSON을 읽는 fixture 계측을 제거했다. 실제 stage marker 및 receipt bytes/magic/길이를 확인하고 원 Resolved 기록의 검증은 public managed.finish에 맡긴다. 잠금 계약을 약화하지 않으며 fixture의 간접/직접 증거 범위를 구분한다.

첫 실제 paired 실행은 metadata/download와 원 receipt 확인을 지난 뒤 fixture의 duration/privacy 묶음 assertion에서 실패했다. 반환 duration은 core의 f64 표현이므로 JSON integer 직접 비교를 as_f64==10.0으로 고쳤고 privacy와 duration 오류를 구분했다. 원 실패 로그는 보존하며 이를 실제 비밀 누출로 단정하지 않는다.

실제 paired 성공 인수: windows-paired-public-download-duration-retry에서 공개 Repository의 버전 확인 및 인증 metadata/download를 실행했다. 한 원 Job의 두 journal·receipt 확인, 인증 출력 생략, 원 cookie/hash·sentinel 보존, 원 소유 기록 정리·재사용을 통과했다. 주 담당이 job_json의 Downloaded/artifact, DB/WAL/SHM의 합성 SECRET 부재, 원 cookie SHA, cookie cache에 foreign sentinel만 남음, managed 9파일/pending 부재와 현재 소스/고정 binary SHA를 독립 대조했다. 실패·취소·부모 crash 및 startup 자동 회수는 아직 후속이다. 마지막 전체 Windows GNU CPU fmt/clippy/test/release도 windows-paired-native-fixture-workspace에서 통과했다.
