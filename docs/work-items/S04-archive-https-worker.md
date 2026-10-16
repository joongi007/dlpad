# S04: archive HTTPS worker 최소 공유 계약

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R05, R08, R11
- 담당/수정 범위: metrics_recovery — 이 문서만; 예정 root direct_download 공유 transport/refactor 및 archive_install adapter, managed 공급/UI 별도 담당
- 선행 작업: package 설치·전체검증·제거 최신 통합, 정확 component 공급 evidence/trust 채택

## 사용자 결과

기존 도구 화면의 고정 공급 후보에서 package를 HTTPS로 내려받아 같은 bounded transport와 원 소유 staging에 검증하고 설치한다. 임의 manifest 선택이나 전체 archive 메모리 읽기를 도입하지 않는다. 실패·취소는 이전 활성 도구를 유지한다.

## 현재 재사용 가능한 계약

direct_download.rs의 DownloadTransport::open와 HttpsTransport는 streaming Box<dyn Read>, HTTPS only, proxy off, redirects5, header64KiB, input64KiB/output16KiB, idleconnection0, remaining globaltimeout 및 resolve/connect/response10초 상한을 제공한다. source_policy는 HTTPS/no userinfo/query/fragment/whitespace를 검사한다. transfer loop는64KiB, expected+1 overflow 검사, 각 read 전후 cancel/deadline, streamSHA, size exact, sync 및1MiB durable staging evidence checkpoint다. acquire_install_lock/capacity/reserve/OwnedDirectory native pin과 원 active 보존을 그대로 공유해야 한다.

## 최소 공유 API 제안

root가 DownloadTransport/HttpsTransport/source_policy/checkpoint를 tools-private reusable transport module 또는 pub(super)로 제한 공개한다. 공개 앱 API는 download_archive_and_install(package:ArchivePackage,cancel:&Arc<AtomicBool>)->Result<InstalledPackage,ToolError> 하나다. source/bytes/hash는 검증된 package에만 따르며 사용자가 임의 URL을 공급하지 않는다. 내부 transfer_to_owned_file(url,expected_bytes,expected_sha256,&mut File,cancel,absolute_deadline,on_checkpoint)는 직접 executable와 archive 양쪽에서 재사용하고 finalsize/hash를 반환한다. checkpoint callback이 기존 journal evidence를 durable 갱신하며 transport가 root/path 삭제를 소유하지 않는다.

archive_install의 현 install_verified_archive(Path,package,&AtomicBool)는 새120초 기한과 별도 lock을 생성한다. 그대로 호출하면 다운로드120초+추출120초, lock 재진입/중복 capacity 예약과 source 경로 reopen이 발생한다. 공유 내부 install_verified_archive_from_file(originalFile,package,cancel,absolute_deadline,held_install_authority)를 분리해 한 worker의 동일 절대 기한/원 source capability/한 lock을 유지한다. 외부 로컬 import API는 기존 wrapper로 보존한다. archive temporary staging File을 source로 넘겨 재다운로드나 Vec read를 하지 않는다. SHA 중복 읽기 자체는 비용으로 측정하되 다른 단계 검증을 무조건 생략하지 않는다.

## 구체 잔여 gate

현재 local install_verified_archive는 implicit activation을 하지 않고 runtime 실행/version proof 및 final-publication crash recovery를 별도 gate로 명시한다. HTTPS adapter는 component 설치만으로 active를 변경하면 안 된다. Deno --version과 FFmpeg/ffprobe 배포 세트 version/codec 호환성을 owned argv/process timeout으로 검증한 뒤 명시 select_package를 별도 수행한다. notice/license bytes는 정확 component hash로 고정하고 보관해야 한다. 기존 reservation은 archive+expanded+두 metadata 및 durable journal 공간을 합산하고 기존 owner/final을 덮어쓰지 않는다. 취소/부분read/timeout/hash/version/publication/DB 실패에서는 소유 staging만 정리하거나 미확인 원 identity를 보존한다.

ureq HTTPS only는 TLS downgrade를 막지만 redirect target host allowlist나 고정tag finalURL 확인은 현 명시 계약에 없다. 원 content SHA는 다른 bytes를 거절하나 공급 metadata 자체의 신뢰를 대신하지 않는다. 고정 GitHub release→공식 CDN redirect 정책/최종 URL 증거를 root 공급 계약으로 결정해야 한다. 개별 blocking read 내부 cancel 즉시성은 없고 bounded transport timeout을 따르므로 실제 slowbody/cancel latency 인수가 필요하다.

## 공급 trust의 현재 범위

managed.rs는 yt-dlp2026.08.19 플랫폼별 executable bytes/hash/HTTPS tag/license를 고정한다. 기존 공급 문서는 expected GPG fingerprint의 checksum VALIDSIG와 변조 BADSIG를 확인했지만 같은 origin의 key import 및 TRUST_UNDEFINED여서 독립 key identity 인증과 구분한다. Deno v2.9.7 공식 API/sha256sum archive bytes/hash 및 MIT 출처는 기록되어 있고 Windows/Linux/macOS component version/notice/runtime 비용·서명/attestation 전체 채택은 남는다. 이후 실제 archive runner 결과는 실행 파일 추출 SHA/비용 증거이지 독립 publisher 인증이 아니다. FFmpeg BtbN LGPL 후보의 공급·buildoptions/license/source/notice/codec 결정은 미확정이다. Proposed를 공식 채택 완료로 표현하지 않는다.

## 인수와 비용

합성 streaming transport로 exact/short/extra/중간오류/slowread/cancel/deadline/redirect-policy 실패에서 source·원 active·foreign sentinel 보존을 검사한다. 실제 고정 HTTPS Deno archive는 root만 다운로드하여 stream hash/원fileidentity/풀린 component hash/notice/version/설치합계/elapsed/RSS를 기록한다. 기존 직접 yt-dlp 설치 회귀를 유지한다. GUI install busy/cancel/retry/oldactive 상태는 사설 desktop으로 검증한다. 시험 서버는 승인된 loopback temporary fixture만이며 제품 웹 UI로 확대하지 않는다.

## 검증과 handoff

managed/direct_download/archive_install/INTEGRATIONS/공급 evidence를 읽어 문서만 작성했다. Rust/shared 문서/Cargo/프로세스/다운로드는 수정·실행하지 않았다. sourcefreeze를 유지한다. 구현 담당·private transport API·version 활성화 정책을 root가 조정한 뒤 수직 구현한다.

## from-file adapter 구현 인계

metrics_recovery는 archive_install.rs만 추가 소유하여 public local wrapper를 보존하고 tools-private install_verified_archive_from_file(&mut self,source_path:&Path,input:&File,package:ArchivePackage,cancel:&AtomicBool,deadline:Instant)를 분리했다. public wrapper는 원 input native open과 동일root install lock을 보유한 뒤 호출한다. 내부는 lock을 다시 얻지 않으며 원 source path/nativeidentity/hash, cancel/동일절대deadline, capacity/reserve/staging evidence/cleanup/publication/최종검증 정책을 유지한다. 호출자는 원 File 및 root lock 수명을 전체 설치 동안 유지해야 한다. 내부 API 자체는 HTTPS 또는 활성 version 증거가 아니다. 담당파일 개별 rustfmt만 수행했고 Cargo/native는 root 통합 검증을 기다린다.

## HTTPS adapter 수직 회귀 작성

기존 archive_install의 stored/multi ZIP fixture를 재사용해 HTTPS memory transport exact 설치(기존 active 유지·source/foreign 보존·stage 없음), short/extra/transport error/hash/cancel/deadline/checkpoint DB 거절의 정확 오류 및 동일 보존, HTTP/userinfo source 정책 transport 호출0 회귀 세 개를 추가했다. 실제 network나 runtime version 증거는 아니다. Cargo는 root 병렬 통합 검증을 위해 실행하지 않았고 새 회귀 성공은 아직 미확인이다.

## 통합 구현과 실제 도구 회귀

root가 archive_download.rs를 추가하고 direct_download.rs의 기존64KiB transfer loop를 공통 helper로 분리했다. 공개 download_archive_and_install은 고정 검증 package/동일120초 전체기한/원 owned source File/한 root lock을 사용하며 inactive로 설치한다. archive+expanded+metadata를 선예약하고 실제 stage inventory 상한을 다시 확인한다. 기존 direct executable의 최종 버전·digest 검증은 유지했다. 새 dependency/Cargo/schema 변경은 없다.

첫 compile은 최종 direct digest용 buffer scope 누락으로 두OS 실패했으며 bounded64KiB 버퍼를 복원했다. Linux 도구 회귀56PASS/1ignored, [Windows 회귀](../validation/evidence/managed-archive-https-native-tools-buffer-fix/tools-test.log)38PASS/0ignored를 root가 실제 확인했다. MemoryTransport exact/stored/multi 및 short/extra/open error/hash/cancel/deadline/DBcheckpoint/sourcepolicy 실패를 검사한다. 실제HTTPS 네트워크/redirect/slowbody/중간read·runtimeversion/license/trust·GUI·전체workspace는 미검증이다. source policy와 TLS의 현 제한을 유지했으며 publisher 승인 또는 공식 공급 채택으로 표현하지 않는다.

## 실행 중 수신 실패 회귀 보강

새 한 회귀에서 실제8바이트 첫 read 후 두 번째 read io error, 첫 read 도중 cancel flag,40ms 전체기한의 첫 read50ms 지연을 각각 요구한다. read 호출 수2/1/1을 확인하여 preflight 실패로 시험이 지나가지 못하게 한다. 정확 Io(Other)/Cancelled/TimedOut 및 이전 active·외부 source·foreign sentinel 보존/목적지·stage 부재를 검증한다. 합성 synchronous reader50ms 실험이며 실제 HTTPS timeout/runtime 증거로 확대하지 않는다. Cargo는 수행하지 않고 root 검증을 기다린다.

## Windows 실행 중 timeout 시험 교정

최신 실제 Windows 실행에서 최초 slow40ms 예산이 원 staging 준비 중 소진되어 reads0으로 엄격히 실패했다. 해당 원시 실패는 보존한다. slow 시험의 전체 worker 예산을2초로 두고 DownloadTransport::open이 받은 동일 절대기한의 remaining을 reader 내부 Instant로 캡처한다. 첫 실제 read에서 그 남은 시간+20ms만 기다려 기존 worker deadline을 넘긴다. reads1과 정확 TimedOut은 그대로이며 pre-read timeout을 허용하지 않는다. 한 slowcase 비용은 약2초이며 기존50ms 문구는 최초 실패 snapshot 설명이다. 실제 HTTPS 성능으로 확대하지 않는다. 담당파일 포맷만 수행/Cargo미실행.

## 현재 전체 검증 범위

Windows GNU CPU workspace capability/fmt/Clippy/test/release terminal0 및 현재sourceSHA370개 일치를 root가 확인했다. [원 전체 결과](../validation/evidence/managed-archive-owned-native-workspace-first/result.json). Linux tools57PASS/1ignored이며 전체collector release진행중이다. 새 midstream exact read regression은 Windows39PASS다. 실제 Linux 공식HTTPS install/remove는 [별도 실증](S04-archive-owned-install-experiment.md)에서 PASS이며 network fixture 전체오류·Windows공식runtime·GUI·publishernotice/trust gate는남는다.
