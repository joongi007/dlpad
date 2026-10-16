# S04: 관리 Deno owned environment 공개 다운로드 fixture staging

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R04, R05, R08, R11, R12
- 담당/수정 파일: metrics_recovery — `/tmp/dlpad-managed-deno-environment-download-fixture.rs`, 이 문서
- 선행 조건: public prepare_managed_deno_environment factory 게시 및 DownloadSession.runtime_environment 공유 연결, fresh native helper/example 빌드

## 범위와 인수 조건

프로젝트 Rust/Cargo/native 실행 없이 새 example 사본을 staged 작성한다. 원 executable 자체의 SHA/bytes와 synthetic notice를 full archive-package로 구성한 managed Deno를 공개 factory와 DownloadRepository session으로 실행한다. managed-only/paired-cookie/wrong Deno version/metadata failure/cancel 다섯 실제 실행 case를 후속 native runner에 제공한다. 실제 환경 이름·값 전체를 출력하지 않고 원 DENO_DIR/고정 shim의 exact 일치만 검사한다. yt-dlp fixture의 직접 Deno child도 원 Tool Job에 귀속된 argv-only 실제 subprocess로 검증하며 nested managed guardian은 생성하지 않는다.

원 source/notice/private cookie/sentinel SHA, 원 cache owner bytes 및 실제 receipts, execution finish/reuse를 검증한다. 외부 Deno policy None/no cache authority 적용은 명시적으로 구분한다. 실제 Deno/EJS/site/영상/network는 시험 범위가 아니며 sourcefixture와 mocked native child만으로 완료를 주장하지 않는다. 실패 root와 원시 evidence는 소유 temp에서 보존한다.

## staged 구현과 실행 계약

`windows_managed_runtime_download_fixture.rs`를 기반으로 새 standalone example 사본을 작성했다. 기존 프로젝트 example은 변경하지 않았다. CLI는 `--fixture-run fresh-result.json --scenario success|paired-cookie|version-mismatch|metadata-failure|cancel`이며 각 case는 새 합성 temp root를 만들고 실패 root도 보존한다. 자기 binary를 yt-dlp/managed Deno/external FFmpeg로 복사하되 실제 Deno나 FFmpeg 실행으로 표현하지 않는다. guardian_entry/self registration은 기존 example과 같이 fresh native helper 코드에 의존한다.

managed Deno는 actual copied executable size/SHA 및 synthetic LICENSE size/SHA가 포함된 full ArchivePackage/InstallManifest를 저장하고 공개 package_installed로 full 검증한다. 외부 Deno copied binary의 공개 factory는 None/cache 생성 없음이어야 하며 실제 managed factory는 Some을 반환해야 한다. 공개 session은 원 policy를 빌려 전달한다. 실제 env 값을 출력하지 않고 Deno version과 yt-dlp metadata/download에서 원 owned cache canonical path 및 shim `1`을 정확 비교한다.

yt-dlp 합성 reader는 같은 원 Tool Job에 직접 native Deno child를 argv-only로 spawn한다. stdin/stdout/stderr는 null이며 nested guardian/managed journal을 만들지 않는다. 원 Child::try_wait 절대 5초, 실패 kill 뒤 원 Child 최대 2초 reap을 적용한다. 이것은 짧은 fixture 전용 비용이며 제품 timeout 추가가 아니다. Deno child는 실제 상속된 환경을 비교하고 stage별 create-new/sync marker를 남긴다. parent cancel은 download가 원 env/child/private cookie를 확인하고 create-new/sync한 `cancel-ready` 뒤에만 발생한다.

paired-cookie/metadata-failure/cancel은 실제 private Netscape import capability를 사용한다. 실제 cookie read는 원 private cache 하위·1 MiB 상한·합성 정확 bytes를 검증하며 synthetic secret stderr를 발생시킨다. 반환 성공/실패 로그와 metadata에서 해당 secret 부재를 요구한다. metadata failure는 실제 exit 7, cancel은 실제 ProcessOutcome::Cancelled 및 persisted JobState::Cancelled를 요구한다. generic unconfirmed cleanup 실패를 성공으로 허용하지 않는다. version-mismatch는 metadata/download를 시작하지 않아야 하며 version fixture가 실제 고정 환경을 확인한 marker를 별도로 요구한다.

managed 실제 receipt 3/4/5개와 magic/slot/stage를 확인하고 원 evidence 사본을 남긴 뒤 public execution finish 및 fresh begin/finish 재사용을 요구한다. private cookie reader 실제 256-byte receipt 1/2개도 확인하고 소유 lease cleanup 뒤 owner 부재를 확인한다. cache owner SHA와 source cookie/notice/original/copied executables/foreign sentinel bytes 보존을 대조한다. CLI evidence는 16 KiB, hash 입력은 256 MiB 상한으로 stream 처리한다. source cookie는 모두 합성이고 account/network/media 시험은 없다.

## 현재 검증과 인계

실제 실행 명령 `.local-tools/cargo/bin/rustfmt --edition 2024 /tmp/dlpad-managed-deno-environment-download-fixture.rs` — exit 0. staged source 31756 bytes, SHA256 `4524774d63c2c1ee0c260fce38ba0a66f2624b46c94d89a760c6943fd7869ba9`다. 프로젝트 Rust 수정/컴파일/Cargo/native 실행은 0이다. 따라서 다섯 case의 native 성공은 아직 주장하지 않는다.

주 담당은 factory 게시/reexport, `DownloadSession.runtime_environment: Option<&WindowsManagedDenoEnvironment>` 및 discovery/metadata/download의 동일 policy 전달, cookie/managed/paired helper 원 cache binding/lock capability 경로를 연결한 뒤 새 example compile와 실제 private host 다섯 case를 검증해야 한다. 실제 Deno payload/cache 생성·EJS/site 호환성은 별도 release gate로 남는다. read-only external factory 대조를 외부 Deno 실행 인수로 확대하지 않는다.

## 최초 공개 경로 Windows 실행 결과

[실제 실행 증거](../validation/evidence/windows-managed-deno-product-20261004T121154Z/result.json)에서 fresh helper/example 빌드, factory native3개, 성공·paired 합성 쿠키·버전 불일치·metadata 실패·취소5case가 모두 exit0이다. [주 담당 대조](../validation/evidence/windows-managed-deno-product-20261004T121154Z/root-review.json)는 실제 원 receipt bytes/SHA/magic, cache owner SHA와 pending 부재를 각 소유 fixture에서 다시 확인했다. 실제 Deno/FFmpeg·사이트·GUI 시험은 아니다.

첫 전체 workspace 시험은 fmt 통과 후 새 example의 Clippy cmp_owned/collapsible_if 두 건으로 실패했다. 실패 로그는 windows-managed-deno-product-workspace-20261004T121430Z에 보존했다. 두 문구를 의미 유지 형태로 수정했으며 현재 source의 전체 검증은 새 collector에서 진행 중이다. 앞선5case 결과는 수정 전 snapshot으로 구분하며 최신 전체 완료를 주장하지 않는다.

## 최신 source 재실행 확정

[최신5case](../validation/evidence/windows-managed-deno-product-corrected-20261004T121739Z/result.json)는 fresh helper/example build와 성공·paired-cookie·version-mismatch·metadata-failure·cancel 모두 exit0이다. root-review.json은 각 operation source SHA가 현재 source와 같고 원 receipt magic/256bytes/SHA, owner SHA, pending 부재를 다시 확인했다. 최신 Windows workspace 네 명령 PASS 근거와 결합해 이 합성 제품 연결 인수는 완료다. 실제 Deno/EJS/site·GUI·부모kill·공급/패키징은 후속 P0로 남는다.
