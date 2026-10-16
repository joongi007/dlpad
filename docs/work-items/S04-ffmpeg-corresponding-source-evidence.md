# S04: FFmpeg 대응 source/build evidence

- 상태: completed (source/recipe 확보; 공급 채택 미완료)
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R11
- 담당/수정 범위: soak_recovery_audit — 이 문서 및 새 프로젝트 전용 공급 evidence 파일
- 선행 작업: exact LGPL Linux artifact 2a571b6068 후보

## 계약과 인수

공식 FFmpeg/BtbN 공개 source와 metadata의 exact revision/URL/digest를 수집한다. 기존 파일을 덮어쓰지 않으며 Rust/Cargo/다른 문서/외부 게시를 변경하지 않는다. source 대응과 재현 build/dependency 완비를 분리하고 불확정은 공급 채택 완료로 표현하지 않는다.

## 실제 확보 결과

공식 [FFmpeg commit](https://github.com/FFmpeg/FFmpeg/commit/2a571b606854520cf89804d8030c8b328e621689)의 full SHA는 `2a571b606854520cf89804d8030c8b328e621689`이다. [BtbN 고정 release](https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-09-30-13-08)의 build repository tag는 `6c9aec5fc9a72ec3abedd1fa84db141fa18cf52b`에 연결된다. API의 tag object/FFmpeg commit/tree와 release artifact 이름의 short hash를 대조했다. 일반 n9.0.2 source 대신 이 commit의 source archive를 실제 확보했다.

새 `.local-tools/ffmpeg-correspondence-9je4wjt6/`에 create-only로 보관했다. [수집 manifest](../../.local-tools/ffmpeg-correspondence-9je4wjt6/manifest.json)는 URL/bytes/SHA256/status 및 404를, [archive/recipe manifest](../../.local-tools/ffmpeg-correspondence-9je4wjt6/source-archives-manifest.json)는 archives와 137개 bounded recipe 파일의 digest를 기록한다.

| artifact | bytes | SHA256 |
|---|---:|---|
| [exact FFmpeg source](https://codeload.github.com/FFmpeg/FFmpeg/tar.gz/2a571b606854520cf89804d8030c8b328e621689) | 17336083 | f02fc0809a13f37e3d4e1f74254d07e035a5ef01a6d91e727b915b1764d97540 |
| [exact BtbN recipe source](https://codeload.github.com/BtbN/FFmpeg-Builds/tar.gz/6c9aec5fc9a72ec3abedd1fa84db141fa18cf52b) | 107893 | 81da4f7c878dc0f60e6b8d0049a112e32178e20f025d468fe4e99e77b2dfe0ec |
| exact FFmpeg COPYING.LGPLv3 | 7651 | da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768 |
| exact FFmpeg COPYING.GPLv3 | 35147 | 8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903 |

COPYING.LGPLv3 hash는 실제 binary archive LICENSE.txt와 동일하다. previously missing GPL3 전문도 확보했다. exact FFmpeg LICENSE.md/configure, BtbN README/MIT LICENSE/build.sh/workflow, Linux64 image/toolchain 관련 recipe 및 dependency scripts를 보관했다. 공식 [FFmpeg license 안내](https://ffmpeg.org/legal.html)를 함께 수집했으나 이 자료 확보 자체를 법적 의무 충족 판정으로 표현하지 않는다.

## source 대응과 남은 재현성 간격

fixed BtbN `addins/9.0.sh`는 release/9.0 branch를 고른다. build.sh는 branch를 clone하고 날짜 기반 extra-version을 붙이며 exact FFmpeg commit checkout을 기본적으로 고정하지 않는다. 실제 version string의 2a571b6068과 확보한 full commit의 대응은 확인했지만 recipe를 지금 다시 실행하면 같은 source/bytes가 된다고 보장하지 않는다. workflow의 image/cache 이름만으로 실제 당시 container digest·download cache·compiler/dependency payload를 모두 고정하지 못한다. 아무 build script도 실행하지 않았다.

[dependency declaration inventory](../../.local-tools/ffmpeg-correspondence-9je4wjt6/dependency-declarations.json)는 SCRIPT_REPO/MIRROR/COMMIT/VERSION/SHA256 선언을 읽기만 해서 정리했다. 이 목록에는 variant에 의해 비활성인 script도 포함되며 실제 enabled dependency 목록이나 notice 완비를 뜻하지 않는다. 다음 개발은 기존 실제 buildconf의 enabled libraries와 이 선언을 대조하고 해당 upstream revision별 source/notice를 수집하는 것이다. compiler image digest와 실제 provenance 확보, dependency별 license/notice bundle, source 제공 경로의 장기 보존, notice UI·package 인수는 여전히 남는다. 전체 binary 재현 build/attestation/Windows/macOS 공급은 미검증이다.

## 실행과 안전 경계

처음 sandbox urllib 수집은 DNS 오류 세 건으로 끝났으며 새 `.local-tools/ffmpeg-correspondence-attfjv60/manifest.json`에 실패를 보존했다. 사용자 허용 공개자료 다운로드를 승격 명령으로 수행해 fresh directory에서 성공했다. 각 response timeout25초/4 MiB, archive timeout40초/64 MiB와 streaming64 KiB SHA를 사용했다. recipe 파일은128 KiB 상한으로 읽고 추출 경로를 생성한 단일 파일명에만 매핑했다. 예상 `scripts.d/50-ffmpeg.sh`는404였고 삭제/덮어쓰기 없이 full recipe source로 실제 경로를 확인했다.

기존 source/license/binary/archive를 변경하지 않았다. Rust/Cargo/도구 실행/외부 게시/공개 배포는 없다. 확보 단계는 완료됐고 공급 상태는 Proposed다. 무료 공식자료 수집·dependency mapping·notice 구현은 별도 사용자 승인 없이 계속할 수 있다.
